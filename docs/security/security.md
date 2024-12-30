# Security Guide

This document outlines the security considerations and configurations for the Sonoma Search engine.

## Overview

Security measures are implemented across several layers:
- Network Security
- Authentication & Authorization
- Data Security
- Infrastructure Security
- Monitoring & Auditing

## Network Security

### TLS Configuration
```nginx
# nginx/conf.d/ssl.conf
ssl_protocols TLSv1.2 TLSv1.3;
ssl_prefer_server_ciphers on;
ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:DHE-RSA-AES128-GCM-SHA256:DHE-RSA-AES256-GCM-SHA384;
ssl_session_timeout 1d;
ssl_session_cache shared:SSL:50m;
ssl_session_tickets off;
ssl_stapling on;
ssl_stapling_verify on;
resolver 8.8.8.8 8.8.4.4 valid=300s;
resolver_timeout 5s;
add_header Strict-Transport-Security "max-age=63072000" always;
```

### Firewall Rules
```bash
# firewall.sh
# External access
ufw allow 80/tcp
ufw allow 443/tcp

# Internal services
ufw allow from 10.0.0.0/8 to any port 8000:8005 proto tcp
ufw allow from 10.0.0.0/8 to any port 9200 proto tcp
ufw allow from 10.0.0.0/8 to any port 5432 proto tcp

# Monitoring
ufw allow from 10.0.0.0/8 to any port 9090:9093 proto tcp
ufw allow from 10.0.0.0/8 to any port 3000 proto tcp
```

### Network Policies
```yaml
# kubernetes/network-policies.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: api-gateway-policy
spec:
  podSelector:
    matchLabels:
      app: api-gateway
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - ipBlock:
        cidr: 10.0.0.0/8
    ports:
    - protocol: TCP
      port: 8000
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: searcher
    - podSelector:
        matchLabels:
          app: ranker
```

## Authentication & Authorization

### API Authentication
```rust
// Authentication middleware
pub async fn authenticate(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, Error> {
    let token = credentials.token();
    
    match validate_token(token).await {
        Ok(claims) => {
            req.extensions_mut().insert(claims);
            Ok(req)
        }
        Err(_) => Err(ErrorUnauthorized("Invalid token")),
    }
}

// Token validation
async fn validate_token(token: &str) -> Result<Claims, Error> {
    let key = jsonwebtoken::DecodingKey::from_secret(
        std::env::var("JWT_SECRET").unwrap().as_bytes()
    );
    
    let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
    
    match jsonwebtoken::decode::<Claims>(token, &key, &validation) {
        Ok(token_data) => Ok(token_data.claims),
        Err(_) => Err(ErrorUnauthorized("Invalid token")),
    }
}
```

### Role-Based Access Control
```rust
// RBAC configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    exp: usize,
    roles: Vec<String>,
}

// Authorization middleware
pub fn authorize(required_roles: Vec<String>) -> impl Fn(ServiceRequest) -> Future<Output = Result<ServiceRequest, Error>> {
    move |req: ServiceRequest| {
        let claims = req.extensions().get::<Claims>().cloned();
        
        async move {
            match claims {
                Some(claims) => {
                    if claims.roles.iter().any(|role| required_roles.contains(role)) {
                        Ok(req)
                    } else {
                        Err(ErrorForbidden("Insufficient permissions"))
                    }
                }
                None => Err(ErrorUnauthorized("Missing authentication")),
            }
        }
    }
}
```

## Data Security

### Encryption at Rest
```yaml
# elasticsearch/elasticsearch.yml
xpack.security.enabled: true
xpack.security.transport.ssl.enabled: true
xpack.security.transport.ssl.verification_mode: certificate
xpack.security.transport.ssl.keystore.path: elastic-certificates.p12
xpack.security.transport.ssl.truststore.path: elastic-certificates.p12

# postgresql/postgresql.conf
ssl = on
ssl_cert_file = '/etc/ssl/certs/postgres.crt'
ssl_key_file = '/etc/ssl/private/postgres.key'
ssl_ca_file = '/etc/ssl/certs/ca.crt'
```

### Data Sanitization
```rust
// Input sanitization
pub fn sanitize_input(input: &str) -> String {
    let mut sanitized = input.to_owned();
    
    // Remove potential SQL injection characters
    sanitized = sanitized.replace(['\'', '"', ';', '-', '='], "");
    
    // Remove potential XSS characters
    sanitized = sanitized.replace(['<', '>', '&'], "");
    
    // Limit length
    sanitized.truncate(1000);
    
    sanitized
}

// Query parameter validation
#[derive(Deserialize, Validate)]
pub struct SearchQuery {
    #[validate(length(min = 1, max = 100))]
    query: String,
    
    #[validate(range(min = 1, max = 100))]
    limit: usize,
    
    #[validate(range(min = 0, max = 1000))]
    offset: usize,
}
```

### Secrets Management
```rust
// Secrets configuration
use std::env;
use dotenv::dotenv;

pub struct Secrets {
    pub database_url: String,
    pub elasticsearch_url: String,
    pub jwt_secret: String,
    pub api_keys: Vec<String>,
}

impl Secrets {
    pub fn from_env() -> Self {
        dotenv().ok();
        
        Self {
            database_url: env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
            elasticsearch_url: env::var("ELASTICSEARCH_URL")
                .expect("ELASTICSEARCH_URL must be set"),
            jwt_secret: env::var("JWT_SECRET")
                .expect("JWT_SECRET must be set"),
            api_keys: env::var("API_KEYS")
                .expect("API_KEYS must be set")
                .split(',')
                .map(String::from)
                .collect(),
        }
    }
}
```

## Infrastructure Security

### Container Security
```dockerfile
# Dockerfile
FROM rust:1.70-slim as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/app/target/release/service /usr/local/bin/
USER nobody
ENTRYPOINT ["service"]
```

### Security Headers
```rust
// Security headers middleware
pub fn security_headers() -> impl Fn(ServiceRequest) -> Future<Output = Result<ServiceRequest, Error>> {
    move |mut req: ServiceRequest| {
        let headers = req.headers_mut();
        
        headers.insert(
            "X-Content-Type-Options",
            HeaderValue::from_static("nosniff"),
        );
        headers.insert(
            "X-Frame-Options",
            HeaderValue::from_static("DENY"),
        );
        headers.insert(
            "X-XSS-Protection",
            HeaderValue::from_static("1; mode=block"),
        );
        headers.insert(
            "Content-Security-Policy",
            HeaderValue::from_static("default-src 'self'"),
        );
        
        async move { Ok(req) }
    }
}
```

## Monitoring & Auditing

### Audit Logging
```rust
// Audit log format
#[derive(Debug, Serialize)]
struct AuditLog {
    timestamp: DateTime<Utc>,
    event_type: String,
    user_id: Option<String>,
    action: String,
    resource: String,
    status: String,
    client_ip: String,
    user_agent: String,
    request_id: String,
}

// Audit logging middleware
pub async fn audit_log(
    req: ServiceRequest,
    next: Next<impl Handler>,
) -> Result<ServiceResponse, Error> {
    let start_time = Utc::now();
    let request_id = Uuid::new_v4().to_string();
    
    let response = next.call(req).await?;
    
    let audit_log = AuditLog {
        timestamp: Utc::now(),
        event_type: "api_request".to_string(),
        user_id: response.request().extensions().get::<Claims>()
            .map(|claims| claims.sub.clone()),
        action: response.request().method().to_string(),
        resource: response.request().path().to_string(),
        status: response.status().as_u16().to_string(),
        client_ip: response.request().connection_info().realip_remote_addr()
            .unwrap_or("unknown").to_string(),
        user_agent: response.request().headers()
            .get("User-Agent")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown")
            .to_string(),
        request_id,
    };
    
    // Log audit event
    log::info!("audit: {}", serde_json::to_string(&audit_log).unwrap());
    
    Ok(response)
}
```

### Security Monitoring
```rust
// Security metrics
lazy_static! {
    static ref FAILED_AUTH_ATTEMPTS: Counter = register_counter!(
        "failed_auth_attempts_total",
        "Total number of failed authentication attempts"
    ).unwrap();
    
    static ref SUSPICIOUS_REQUESTS: Counter = register_counter!(
        "suspicious_requests_total",
        "Total number of suspicious requests detected",
        &["type"]
    ).unwrap();
    
    static ref RATE_LIMIT_EXCEEDED: Counter = register_counter!(
        "rate_limit_exceeded_total",
        "Total number of rate limit exceeded events",
        &["client_ip"]
    ).unwrap();
}

// Rate limiting
pub struct RateLimiter {
    store: Arc<RwLock<HashMap<String, (Instant, u32)>>>,
    max_requests: u32,
    window_size: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_size: Duration) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            max_requests,
            window_size,
        }
    }
    
    pub async fn check(&self, key: &str) -> bool {
        let now = Instant::now();
        let mut store = self.store.write().await;
        
        match store.get(key) {
            Some((window_start, count)) => {
                if now.duration_since(*window_start) > self.window_size {
                    store.insert(key.to_string(), (now, 1));
                    true
                } else if *count >= self.max_requests {
                    RATE_LIMIT_EXCEEDED.with_label_values(&[key]).inc();
                    false
                } else {
                    store.insert(key.to_string(), (*window_start, count + 1));
                    true
                }
            }
            None => {
                store.insert(key.to_string(), (now, 1));
                true
            }
        }
    }
}
```

## Security Best Practices

### Password Hashing
```rust
use argon2::{self, Config};

pub async fn hash_password(password: &str) -> Result<String, Error> {
    let salt = rand::random::<[u8; 32]>();
    let config = Config::default();
    
    argon2::hash_encoded(
        password.as_bytes(),
        &salt,
        &config,
    )
    .map_err(|_| ErrorInternalServerError("Failed to hash password"))
}

pub async fn verify_password(password: &str, hash: &str) -> Result<bool, Error> {
    argon2::verify_encoded(hash, password.as_bytes())
        .map_err(|_| ErrorInternalServerError("Failed to verify password"))
}
```

### API Rate Limiting
```rust
// Rate limiting middleware
pub struct RateLimitingMiddleware {
    limiter: Arc<RateLimiter>,
}

impl RateLimitingMiddleware {
    pub fn new(max_requests: u32, window_size: Duration) -> Self {
        Self {
            limiter: Arc::new(RateLimiter::new(max_requests, window_size)),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimitingMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RateLimitingMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimitingMiddlewareService {
            service,
            limiter: self.limiter.clone(),
        }))
    }
}
```

### Security Headers Configuration
```rust
// Configure security headers
pub fn configure_security_headers(config: &mut web::ServiceConfig) {
    config.wrap_fn(|req, srv| {
        let mut response = srv.call(req);
        response.map(|mut res| {
            let headers = res.headers_mut();
            
            headers.insert(
                header::STRICT_TRANSPORT_SECURITY,
                HeaderValue::from_static("max-age=31536000; includeSubDomains"),
            );
            headers.insert(
                header::X_CONTENT_TYPE_OPTIONS,
                HeaderValue::from_static("nosniff"),
            );
            headers.insert(
                header::X_FRAME_OPTIONS,
                HeaderValue::from_static("DENY"),
            );
            headers.insert(
                header::X_XSS_PROTECTION,
                HeaderValue::from_static("1; mode=block"),
            );
            headers.insert(
                header::CONTENT_SECURITY_POLICY,
                HeaderValue::from_static(
                    "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; font-src 'self';"
                ),
            );
            
            res
        })
    });
}
``` 