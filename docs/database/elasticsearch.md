# Elasticsearch Setup

This document describes the Elasticsearch setup and configuration for the Sonoma Search engine.

## Index Configuration

### Document Index
```json
{
  "settings": {
    "index": {
      "number_of_shards": 5,
      "number_of_replicas": 2,
      "refresh_interval": "1s",
      "analysis": {
        "analyzer": {
          "html_strip": {
            "tokenizer": "standard",
            "filter": ["lowercase", "stop", "snowball"],
            "char_filter": ["html_strip"]
          },
          "ngram_analyzer": {
            "tokenizer": "ngram_tokenizer",
            "filter": ["lowercase"]
          }
        },
        "tokenizer": {
          "ngram_tokenizer": {
            "type": "ngram",
            "min_gram": 3,
            "max_gram": 4,
            "token_chars": ["letter", "digit"]
          }
        }
      }
    }
  },
  "mappings": {
    "properties": {
      "url": {
        "type": "keyword"
      },
      "title": {
        "type": "text",
        "analyzer": "html_strip",
        "fields": {
          "ngram": {
            "type": "text",
            "analyzer": "ngram_analyzer"
          },
          "keyword": {
            "type": "keyword"
          }
        }
      },
      "content": {
        "type": "text",
        "analyzer": "html_strip",
        "fields": {
          "ngram": {
            "type": "text",
            "analyzer": "ngram_analyzer"
          }
        }
      },
      "language": {
        "type": "keyword"
      },
      "content_type": {
        "type": "keyword"
      },
      "crawled_at": {
        "type": "date"
      },
      "last_modified": {
        "type": "date"
      },
      "metadata": {
        "type": "object",
        "dynamic": true
      },
      "pagerank": {
        "type": "float"
      },
      "inbound_links": {
        "type": "integer"
      },
      "outbound_links": {
        "type": "integer"
      }
    }
  }
}
```

## Cluster Configuration

### Node Settings
```yaml
# elasticsearch.yml
cluster.name: sonoma-search
node.name: ${HOSTNAME}
node.roles: [master, data]

# Network
network.host: 0.0.0.0
http.port: 9200
transport.port: 9300

# Discovery
discovery.seed_hosts: ["es01:9300", "es02:9300", "es03:9300"]
cluster.initial_master_nodes: ["es01", "es02", "es03"]

# Memory
bootstrap.memory_lock: true

# Security
xpack.security.enabled: true
xpack.security.transport.ssl.enabled: true
```

### JVM Settings
```bash
# jvm.options
-Xms4g
-Xmx4g
-XX:+UseG1GC
-XX:G1ReservePercent=25
-XX:InitiatingHeapOccupancyPercent=30
```

## Search Configuration

### Search Templates
```json
{
  "script": {
    "lang": "mustache",
    "source": {
      "query": {
        "bool": {
          "must": [
            {
              "multi_match": {
                "query": "{{query_string}}",
                "fields": ["title^3", "content"],
                "type": "best_fields",
                "tie_breaker": 0.3
              }
            }
          ],
          "should": [
            {
              "term": {
                "language": {
                  "value": "{{preferred_language}}",
                  "boost": 1.2
                }
              }
            }
          ],
          "filter": [
            {
              "range": {
                "crawled_at": {
                  "gte": "now-6M"
                }
              }
            }
          ]
        }
      },
      "size": "{{size}}",
      "from": "{{from}}",
      "_source": ["url", "title", "content", "crawled_at", "metadata"],
      "highlight": {
        "fields": {
          "title": {},
          "content": {
            "fragment_size": 150,
            "number_of_fragments": 3
          }
        }
      }
    }
  }
}
```

## Docker Configuration

```yaml
version: '3.8'

services:
  es01:
    image: docker.elastic.co/elasticsearch/elasticsearch:8.11.1
    container_name: es01
    environment:
      - node.name=es01
      - cluster.name=sonoma-search
      - discovery.seed_hosts=es02,es03
      - cluster.initial_master_nodes=es01,es02,es03
      - bootstrap.memory_lock=true
      - "ES_JAVA_OPTS=-Xms4g -Xmx4g"
    ulimits:
      memlock:
        soft: -1
        hard: -1
    volumes:
      - es01_data:/usr/share/elasticsearch/data
    ports:
      - "9200:9200"
    networks:
      - elastic

  es02:
    image: docker.elastic.co/elasticsearch/elasticsearch:8.11.1
    container_name: es02
    environment:
      - node.name=es02
      - cluster.name=sonoma-search
      - discovery.seed_hosts=es01,es03
      - cluster.initial_master_nodes=es01,es02,es03
      - bootstrap.memory_lock=true
      - "ES_JAVA_OPTS=-Xms4g -Xmx4g"
    ulimits:
      memlock:
        soft: -1
        hard: -1
    volumes:
      - es02_data:/usr/share/elasticsearch/data
    networks:
      - elastic

  es03:
    image: docker.elastic.co/elasticsearch/elasticsearch:8.11.1
    container_name: es03
    environment:
      - node.name=es03
      - cluster.name=sonoma-search
      - discovery.seed_hosts=es01,es02
      - cluster.initial_master_nodes=es01,es02,es03
      - bootstrap.memory_lock=true
      - "ES_JAVA_OPTS=-Xms4g -Xmx4g"
    ulimits:
      memlock:
        soft: -1
        hard: -1
    volumes:
      - es03_data:/usr/share/elasticsearch/data
    networks:
      - elastic

volumes:
  es01_data:
  es02_data:
  es03_data:

networks:
  elastic:
```

## Maintenance Procedures

### Index Management
```bash
# Create index
curl -X PUT "localhost:9200/documents" -H "Content-Type: application/json" -d @index-settings.json

# Delete index
curl -X DELETE "localhost:9200/documents"

# Update settings
curl -X PUT "localhost:9200/documents/_settings" -H "Content-Type: application/json" -d '{
  "index": {
    "refresh_interval": "30s",
    "number_of_replicas": 1
  }
}'
```

### Backup and Restore

```bash
# Register repository
curl -X PUT "localhost:9200/_snapshot/backup" -H "Content-Type: application/json" -d '{
  "type": "fs",
  "settings": {
    "location": "/backup/elasticsearch"
  }
}'

# Create snapshot
curl -X PUT "localhost:9200/_snapshot/backup/snapshot_1?wait_for_completion=true"

# Restore snapshot
curl -X POST "localhost:9200/_snapshot/backup/snapshot_1/_restore"
```

## Monitoring

### Cluster Health
```bash
# Check cluster health
curl -X GET "localhost:9200/_cluster/health?pretty"

# Check nodes info
curl -X GET "localhost:9200/_nodes/stats?pretty"

# Check indices
curl -X GET "localhost:9200/_cat/indices?v"
```

### Performance Monitoring
```bash
# Thread pool stats
curl -X GET "localhost:9200/_nodes/stats/thread_pool?pretty"

# Memory usage
curl -X GET "localhost:9200/_nodes/stats/jvm?pretty"

# Search performance
curl -X GET "localhost:9200/_nodes/stats/indices/search?pretty"
```

## Security Configuration

### User Management
```bash
# Create user
curl -X POST "localhost:9200/_security/user/searcher" -H "Content-Type: application/json" -d '{
  "password": "xxx",
  "roles": ["search_role"]
}'

# Create role
curl -X POST "localhost:9200/_security/role/search_role" -H "Content-Type: application/json" -d '{
  "cluster": ["monitor"],
  "indices": [
    {
      "names": ["documents"],
      "privileges": ["read", "view_index_metadata"]
    }
  ]
}'
```

### SSL Configuration
```yaml
xpack.security.http.ssl:
  enabled: true
  key: certs/http.key
  certificate: certs/http.crt

xpack.security.transport.ssl:
  enabled: true
  verification_mode: certificate
  key: certs/transport.key
  certificate: certs/transport.crt
  certificate_authorities: ["certs/ca.crt"]
```

## Performance Tuning

### Memory Settings
```yaml
# config/jvm.options
-Xms4g
-Xmx4g

# Heap dump settings
-XX:+HeapDumpOnOutOfMemoryError
-XX:HeapDumpPath=/var/log/elasticsearch/heap-dump.hprof

# GC logging
-Xlog:gc*,gc+age=trace,safepoint:file=/var/log/elasticsearch/gc.log:utctime,pid,tags:filecount=32,filesize=64m
```

### Index Performance
```json
{
  "index.refresh_interval": "30s",
  "index.number_of_replicas": 1,
  "index.translog.durability": "async",
  "index.translog.sync_interval": "30s",
  "index.merge.scheduler.max_thread_count": 1
}
``` 