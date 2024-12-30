# Architecture Overview

Sonoma Search is built using a microservices architecture, designed for scalability, maintainability, and reliability. The system is composed of several independent services that work together to provide a comprehensive search solution.

## System Architecture

```mermaid
%%{init: {
  'theme': 'base',
  'themeVariables': {
    'fontFamily': 'Arial',
    'fontSize': '16px',
    'primaryColor': '#89DCEB',
    'primaryTextColor': '#ffffff',
    'primaryBorderColor': '#74C7EC',
    'lineColor': '#CBA6F7',
    'secondaryColor': '#F5C2E7',
    'tertiaryColor': '#ABE9B3',
    'labelBackground': '#00000000',
    'labelTextColor': '#ffffff',
    'edgeLabelBackground': '#00000000',
    'nodeTextColor': '#ffffff'
  }
}}%%
flowchart TD
    %% Frontend Layer
    subgraph Frontend["Frontend Layer"]
        A[Web Frontend]-->|"HTTP Requests"|B[API Gateway]
        style A fill:#F5C2E7,stroke:#181825,color:#ffffff
        style B fill:#89DCEB,stroke:#181825,color:#ffffff
    end
    
    %% Search Layer
    subgraph Search["Search Layer"]
        B-->|"Search Requests"|C[Searcher Service]
        B-->|"Ranking Requests"|D[Ranker Service]
        C-->|"Query"|E[Elasticsearch]
        D-->|"Score"|E
        style C fill:#ABE9B3,stroke:#181825,color:#ffffff
        style D fill:#F9E2AF,stroke:#181825,color:#ffffff
        style E fill:#FAB387,stroke:#181825,color:#ffffff
    end

    %% Data Layer
    subgraph Data["Data Layer"]
        G[Crawler Service]-->|"Fetch"|H[Web Pages]
        G-->|"Store"|I[(PostgreSQL)]
        F[Indexer Service]-->|"Index"|E
        J[Storage Service]-->|"Manage"|I
        G-->|"Trigger"|F
        
        style F fill:#F38BA8,stroke:#181825,color:#ffffff
        style G fill:#94E2D5,stroke:#181825,color:#ffffff
        style H fill:#89DCEB,stroke:#181825,color:#ffffff
        style I fill:#CBA6F7,stroke:#181825,color:#ffffff
        style J fill:#ABE9B3,stroke:#181825,color:#ffffff
    end

    %% Deployment Layer
    subgraph Deployment["Deployment Layer"]
        R[GitHub Actions]-->|"Build"|S[Docker Registry]
        S-->|"Deploy"|A & B & C & D & F & G & J
        style R fill:#ABE9B3,stroke:#181825,color:#ffffff
        style S fill:#F5C2E7,stroke:#181825,color:#ffffff
    end

    %% Monitoring Layer
    subgraph Monitoring["Monitoring Layer"]
        CM["Searcher Metrics<br/>/metrics:9091"] & DM["Ranker Metrics<br/>/metrics:9092"] & FM["Indexer Metrics<br/>/metrics:9093"] & GM["Crawler Metrics<br/>/metrics:9094"] & JM["Storage Metrics<br/>/metrics:9095"]-->P[Prometheus]
        P-->|"Visualize"|Q[Grafana]
        
        C-->|"Expose"|CM
        D-->|"Expose"|DM
        F-->|"Expose"|FM
        G-->|"Expose"|GM
        J-->|"Expose"|JM
        
        style CM fill:#B4BEFE,stroke:#181825,color:#ffffff
        style DM fill:#B4BEFE,stroke:#181825,color:#ffffff
        style FM fill:#B4BEFE,stroke:#181825,color:#ffffff
        style GM fill:#B4BEFE,stroke:#181825,color:#ffffff
        style JM fill:#B4BEFE,stroke:#181825,color:#ffffff
        style P fill:#F38BA8,stroke:#181825,color:#ffffff
        style Q fill:#94E2D5,stroke:#181825,color:#ffffff
    end
```

## Layer Description

### Frontend Layer
- **Web Frontend**: User interface for search interactions
- **API Gateway**: Central entry point for all client requests

### Search Layer
- **Searcher Service**: Handles search queries and retrieval
- **Ranker Service**: Implements search result ranking algorithms
- **Elasticsearch**: Search engine for efficient text search and indexing

### Data Layer
- **Crawler Service**: Fetches and processes web pages
- **Indexer Service**: Processes and indexes documents
- **Storage Service**: Manages data persistence
- **PostgreSQL**: Primary data store

### Deployment Layer
- **GitHub Actions**: CI/CD pipeline
- **Docker Registry**: Container image storage and distribution

### Monitoring Layer
- **Prometheus**: Metrics collection and storage
- **Grafana**: Metrics visualization and alerting
- **Service Metrics**: Each service exposes metrics on dedicated ports

## Communication Flow

1. Users interact with the Web Frontend
2. Requests are routed through the API Gateway
3. Search requests are processed by the Searcher and Ranker services
4. The Crawler continuously fetches new web pages
5. The Indexer processes and indexes new documents
6. The Storage Service manages data persistence
7. All services are monitored through Prometheus and Grafana

## Service Dependencies

- **Frontend → API Gateway**: HTTP/REST
- **API Gateway → Services**: HTTP/REST
- **Services → Elasticsearch**: HTTP/REST
- **Services → PostgreSQL**: Native PostgreSQL protocol
- **Services → Prometheus**: Metrics exposition 