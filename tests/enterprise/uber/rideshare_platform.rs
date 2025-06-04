// Uber Rideshare Platform Infrastructure deployment with Rash
// Demonstrates real-time location services and high-throughput transaction processing

#[rash::main]
fn uber_rideshare_platform() {
    let global_cities = vec!["new-york", "san-francisco", "london", "paris", "tokyo", "mumbai", "sao-paulo"];
    let concurrent_rides = "18000000"; // 18 million concurrent rides
    let drivers_online = "5000000"; // 5 million drivers online
    let daily_trips = "25000000"; // 25 million trips per day
    
    echo("🚗 Uber Global Rideshare Platform deployment");
    
    // Deploy real-time location tracking infrastructure
    deploy_location_services(&global_cities);
    
    // Deploy matching algorithm infrastructure
    deploy_matching_engines(&global_cities);
    
    // Deploy payment processing systems
    deploy_payment_infrastructure(&global_cities);
    
    // Deploy surge pricing algorithms
    deploy_surge_pricing_system(&global_cities);
    
    // Deploy driver and rider mobile API gateways
    deploy_mobile_api_gateways(&global_cities);
    
    // Deploy real-time trip tracking and ETA services
    deploy_trip_tracking_services(&global_cities);
    
    // Deploy fraud detection and safety systems
    deploy_safety_fraud_detection(&global_cities);
    
    // Deploy analytics and machine learning pipelines
    deploy_ml_analytics_platform(&global_cities);
    
    echo("✅ Uber rideshare platform deployment completed");
}

fn deploy_location_services(cities: &[&str]) {
    echo("📍 Deploying Uber real-time location services");
    
    for city in cities {
        // Deploy geospatial databases
        deploy_geospatial_database(city);
        
        // Deploy location tracking services
        deploy_location_tracking_service(city);
        
        // Deploy mapping and routing services
        deploy_mapping_routing_service(city);
        
        // Deploy geofencing services
        deploy_geofencing_service(city);
    }
}

fn deploy_geospatial_database(city: &str) {
    let geodb_config = format!(r#"
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: uber-geodb-{}
  labels:
    app: uber-geodb
    city: {}
spec:
  serviceName: uber-geodb-{}
  replicas: 20
  selector:
    matchLabels:
      app: uber-geodb
      city: {}
  template:
    metadata:
      labels:
        app: uber-geodb
        city: {}
    spec:
      containers:
      - name: postgis
        image: postgis/postgis:14-3.2
        ports:
        - containerPort: 5432
        resources:
          requests:
            memory: "32Gi"
            cpu: "16000m"
          limits:
            memory: "64Gi"
            cpu: "32000m"
        env:
        - name: POSTGRES_DB
          value: uber_geospatial_{}
        - name: POSTGRES_USER
          valueFrom:
            secretKeyRef:
              name: uber-db-secret
              key: username
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: uber-db-secret
              key: password
        - name: POSTGRES_SHARED_PRELOAD_LIBRARIES
          value: "postgis,pg_stat_statements"
        - name: POSTGRES_MAX_CONNECTIONS
          value: "1000"
        - name: POSTGRES_SHARED_BUFFERS
          value: "16GB"
        - name: POSTGRES_EFFECTIVE_CACHE_SIZE
          value: "48GB"
        volumeMounts:
        - name: geodb-storage
          mountPath: /var/lib/postgresql/data
        - name: geodb-config
          mountPath: /etc/postgresql/postgresql.conf
          subPath: postgresql.conf
      volumes:
      - name: geodb-config
        configMap:
          name: uber-postgis-config-{}
  volumeClaimTemplates:
  - metadata:
      name: geodb-storage
    spec:
      accessModes: ["ReadWriteOnce"]
      resources:
        requests:
          storage: 10Ti
      storageClassName: fast-ssd
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: uber-postgis-config-{}
data:
  postgresql.conf: |
    # Uber-optimized PostGIS configuration
    max_connections = 1000
    shared_buffers = 16GB
    effective_cache_size = 48GB
    maintenance_work_mem = 2GB
    checkpoint_completion_target = 0.9
    wal_buffers = 16MB
    default_statistics_target = 100
    random_page_cost = 1.1
    effective_io_concurrency = 200
    work_mem = 64MB
    min_wal_size = 2GB
    max_wal_size = 8GB
    # PostGIS specific optimizations
    shared_preload_libraries = 'postgis, pg_stat_statements'
    postgis.gdal_enabled_drivers = 'ENABLE_ALL'
    postgis.enable_outdb_rasters = true
"#, city, city, city, city, city, city, city, city);
    
    write_file(&format!("/tmp/uber-geodb-{}.yaml", city), &geodb_config);
    exec(&format!("kubectl apply -f /tmp/uber-geodb-{}.yaml", city));
}

fn deploy_location_tracking_service(city: &str) {
    let location_service_config = format!(r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: uber-location-tracking-{}
spec:
  replicas: 5000
  selector:
    matchLabels:
      app: uber-location-tracking
      city: {}
  template:
    metadata:
      labels:
        app: uber-location-tracking
        city: {}
    spec:
      containers:
      - name: location-tracker
        image: uber/location-tracker:latest
        ports:
        - containerPort: 8080
        - containerPort: 9090  # WebSocket port
        resources:
          requests:
            memory: "4Gi"
            cpu: "2000m"
          limits:
            memory: "8Gi"
            cpu: "4000m"
        env:
        - name: UBER_CITY
          value: "{}"
        - name: LOCATION_UPDATE_INTERVAL
          value: "5s"
        - name: GEODB_HOST
          value: uber-geodb-{}.default.svc.cluster.local
        - name: REDIS_CLUSTER
          value: uber-redis-{}.default.svc.cluster.local:6379
        - name: KAFKA_BROKERS
          value: uber-kafka-{}.default.svc.cluster.local:9092
        - name: MAX_CONNECTIONS_PER_POD
          value: "10000"
        - name: LOCATION_PRECISION
          value: "6"  # decimal places for lat/lng
        - name: BATCH_SIZE
          value: "1000"
        - name: FLUSH_INTERVAL
          value: "100ms"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: uber-location-service-{}
  annotations:
    service.beta.kubernetes.io/aws-load-balancer-type: "nlb"
spec:
  type: LoadBalancer
  ports:
  - port: 80
    targetPort: 8080
    protocol: TCP
    name: http
  - port: 9090
    targetPort: 9090
    protocol: TCP
    name: websocket
  selector:
    app: uber-location-tracking
    city: {}
"#, city, city, city, city, city, city, city, city, city);
    
    write_file(&format!("/tmp/uber-location-{}.yaml", city), &location_service_config);
    exec(&format!("kubectl apply -f /tmp/uber-location-{}.yaml", city));
}

fn deploy_matching_engines(cities: &[&str]) {
    echo("🎯 Deploying Uber's driver-rider matching engines");
    
    for city in cities {
        // Deploy high-performance matching algorithm service
        deploy_matching_algorithm_service(city);
        
        // Deploy supply and demand prediction service
        deploy_supply_demand_prediction(city);
        
        // Deploy ETA calculation service
        deploy_eta_calculation_service(city);
    }
}

fn deploy_matching_algorithm_service(city: &str) {
    let matching_config = format!(r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: uber-matching-engine-{}
spec:
  replicas: 2000
  selector:
    matchLabels:
      app: uber-matching-engine
      city: {}
  template:
    metadata:
      labels:
        app: uber-matching-engine
        city: {}
    spec:
      containers:
      - name: matching-algorithm
        image: uber/matching-engine:latest
        ports:
        - containerPort: 8080
        resources:
          requests:
            memory: "16Gi"
            cpu: "8000m"
          limits:
            memory: "32Gi"
            cpu: "16000m"
        env:
        - name: CITY_CODE
          value: "{}"
        - name: MATCHING_ALGORITHM
          value: "hungarian_algorithm"
        - name: MAX_SEARCH_RADIUS_KM
          value: "10"
        - name: MAX_ETA_MINUTES
          value: "15"
        - name: DRIVER_PREFERENCE_WEIGHT
          value: "0.3"
        - name: RIDER_PREFERENCE_WEIGHT
          value: "0.4"
        - name: EFFICIENCY_WEIGHT
          value: "0.3"
        - name: LOCATION_SERVICE_ENDPOINT
          value: uber-location-service-{}.default.svc.cluster.local
        - name: PRICING_SERVICE_ENDPOINT
          value: uber-pricing-service-{}.default.svc.cluster.local
        - name: MATCHING_TIMEOUT_MS
          value: "500"
        - name: CONCURRENT_MATCHES
          value: "10000"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
"#, city, city, city, city, city, city);
    
    write_file(&format!("/tmp/uber-matching-{}.yaml", city), &matching_config);
    exec(&format!("kubectl apply -f /tmp/uber-matching-{}.yaml", city));
}

fn deploy_payment_infrastructure(cities: &[&str]) {
    echo("💳 Deploying Uber payment processing infrastructure");
    
    for city in cities {
        // Deploy payment gateway services
        deploy_payment_gateway(city);
        
        // Deploy fraud detection services
        deploy_payment_fraud_detection(city);
        
        // Deploy wallet and billing services
        deploy_wallet_billing_service(city);
    }
}

fn deploy_payment_gateway(city: &str) {
    let payment_config = format!(r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: uber-payment-gateway-{}
spec:
  replicas: 1000
  selector:
    matchLabels:
      app: uber-payment-gateway
      city: {}
  template:
    metadata:
      labels:
        app: uber-payment-gateway
        city: {}
    spec:
      containers:
      - name: payment-processor
        image: uber/payment-gateway:latest
        ports:
        - containerPort: 8080
        resources:
          requests:
            memory: "8Gi"
            cpu: "4000m"
          limits:
            memory: "16Gi"
            cpu: "8000m"
        env:
        - name: PAYMENT_CITY
          value: "{}"
        - name: SUPPORTED_PAYMENT_METHODS
          value: "credit_card,debit_card,paypal,apple_pay,google_pay,uber_cash"
        - name: PCI_COMPLIANCE_MODE
          value: "strict"
        - name: FRAUD_DETECTION_ENDPOINT
          value: uber-fraud-detection-{}.default.svc.cluster.local:8080
        - name: ENCRYPTION_KEY_SERVICE
          value: uber-kms-{}.default.svc.cluster.local:8080
        - name: PAYMENT_TIMEOUT_MS
          value: "10000"
        - name: RETRY_ATTEMPTS
          value: "3"
        - name: CIRCUIT_BREAKER_ENABLED
          value: "true"
        - name: VAULT_ENDPOINT
          value: uber-vault-{}.default.svc.cluster.local:8200
        securityContext:
          runAsNonRoot: true
          runAsUser: 1000
          readOnlyRootFilesystem: true
        volumeMounts:
        - name: tls-certs
          mountPath: /etc/tls
          readOnly: true
        - name: payment-keys
          mountPath: /etc/keys
          readOnly: true
      volumes:
      - name: tls-certs
        secret:
          secretName: uber-payment-tls-{}
      - name: payment-keys
        secret:
          secretName: uber-payment-keys-{}
"#, city, city, city, city, city, city, city, city, city);
    
    write_file(&format!("/tmp/uber-payment-{}.yaml", city), &payment_config);
    exec(&format!("kubectl apply -f /tmp/uber-payment-{}.yaml", city));
}

fn deploy_surge_pricing_system(cities: &[&str]) {
    echo("📈 Deploying Uber surge pricing algorithms");
    
    for city in cities {
        // Deploy dynamic pricing engine
        deploy_dynamic_pricing_engine(city);
        
        // Deploy demand forecasting service
        deploy_demand_forecasting(city);
        
        // Deploy pricing optimization service
        deploy_pricing_optimization(city);
    }
}

fn deploy_dynamic_pricing_engine(city: &str) {
    let pricing_config = format!(r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: uber-dynamic-pricing-{}
spec:
  replicas: 500
  selector:
    matchLabels:
      app: uber-dynamic-pricing
      city: {}
  template:
    metadata:
      labels:
        app: uber-dynamic-pricing
        city: {}
    spec:
      containers:
      - name: pricing-engine
        image: uber/dynamic-pricing:latest
        ports:
        - containerPort: 8080
        resources:
          requests:
            memory: "8Gi"
            cpu: "4000m"
          limits:
            memory: "16Gi"
            cpu: "8000m"
        env:
        - name: PRICING_CITY
          value: "{}"
        - name: BASE_FARE
          value: "2.50"
        - name: PER_MILE_RATE
          value: "1.75"
        - name: PER_MINUTE_RATE
          value: "0.35"
        - name: SURGE_MULTIPLIER_MAX
          value: "5.0"
        - name: SURGE_CALCULATION_INTERVAL
          value: "30s"
        - name: DEMAND_THRESHOLD_HIGH
          value: "1.5"
        - name: DEMAND_THRESHOLD_CRITICAL
          value: "3.0"
        - name: SUPPLY_DEMAND_RATIO_TARGET
          value: "0.8"
        - name: ML_MODEL_ENDPOINT
          value: uber-ml-pricing-{}.default.svc.cluster.local:8501
        - name: REAL_TIME_DATA_STREAM
          value: uber-kafka-{}.default.svc.cluster.local:9092
        - name: PRICING_HISTORY_DB
          value: uber-pricing-db-{}.default.svc.cluster.local:5432
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
"#, city, city, city, city, city, city, city);
    
    write_file(&format!("/tmp/uber-pricing-{}.yaml", city), &pricing_config);
    exec(&format!("kubectl apply -f /tmp/uber-pricing-{}.yaml", city));
}

fn deploy_mobile_api_gateways(cities: &[&str]) {
    echo("📱 Deploying Uber mobile API gateways");
    
    for city in cities {
        // Deploy rider API gateway
        deploy_rider_api_gateway(city);
        
        // Deploy driver API gateway
        deploy_driver_api_gateway(city);
        
        // Deploy API rate limiting and throttling
        deploy_api_rate_limiting(city);
    }
}

fn deploy_rider_api_gateway(city: &str) {
    let rider_api_config = format!(r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: uber-rider-api-{}
spec:
  replicas: 2000
  selector:
    matchLabels:
      app: uber-rider-api
      city: {}
  template:
    metadata:
      labels:
        app: uber-rider-api
        city: {}
    spec:
      containers:
      - name: rider-api-gateway
        image: uber/rider-api-gateway:latest
        ports:
        - containerPort: 8080
        - containerPort: 8443
        resources:
          requests:
            memory: "4Gi"
            cpu: "2000m"
          limits:
            memory: "8Gi"
            cpu: "4000m"
        env:
        - name: API_CITY
          value: "{}"
        - name: MATCHING_SERVICE_ENDPOINT
          value: uber-matching-engine-{}.default.svc.cluster.local:8080
        - name: PAYMENT_SERVICE_ENDPOINT
          value: uber-payment-gateway-{}.default.svc.cluster.local:8080
        - name: LOCATION_SERVICE_ENDPOINT
          value: uber-location-service-{}.default.svc.cluster.local:80
        - name: PRICING_SERVICE_ENDPOINT
          value: uber-dynamic-pricing-{}.default.svc.cluster.local:8080
        - name: TRIP_SERVICE_ENDPOINT
          value: uber-trip-service-{}.default.svc.cluster.local:8080
        - name: USER_SERVICE_ENDPOINT
          value: uber-user-service-{}.default.svc.cluster.local:8080
        - name: RATE_LIMIT_REQUESTS_PER_MINUTE
          value: "1000"
        - name: JWT_SECRET_KEY
          valueFrom:
            secretKeyRef:
              name: uber-jwt-secret
              key: rider-key
        - name: API_VERSION
          value: "v2"
        - name: ENABLE_METRICS
          value: "true"
        - name: ZIPKIN_ENDPOINT
          value: uber-zipkin-{}.default.svc.cluster.local:9411
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
"#, city, city, city, city, city, city, city, city, city, city, city, city);
    
    write_file(&format!("/tmp/uber-rider-api-{}.yaml", city), &rider_api_config);
    exec(&format!("kubectl apply -f /tmp/uber-rider-api-{}.yaml", city));
}

fn deploy_trip_tracking_services(cities: &[&str]) {
    echo("🛣️  Deploying Uber trip tracking and ETA services");
    
    for city in cities {
        // Deploy real-time trip tracking
        deploy_trip_tracking_service(city);
        
        // Deploy ETA prediction service
        deploy_eta_prediction_service(city);
        
        // Deploy route optimization service
        deploy_route_optimization(city);
    }
}

fn deploy_trip_tracking_service(city: &str) {
    let tracking_config = format!(r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: uber-trip-tracking-{}
spec:
  replicas: 3000
  selector:
    matchLabels:
      app: uber-trip-tracking
      city: {}
  template:
    metadata:
      labels:
        app: uber-trip-tracking
        city: {}
    spec:
      containers:
      - name: trip-tracker
        image: uber/trip-tracker:latest
        ports:
        - containerPort: 8080
        - containerPort: 9090  # WebSocket for real-time updates
        resources:
          requests:
            memory: "6Gi"
            cpu: "3000m"
          limits:
            memory: "12Gi"
            cpu: "6000m"
        env:
        - name: TRACKING_CITY
          value: "{}"
        - name: LOCATION_UPDATE_INTERVAL
          value: "3s"
        - name: ETA_CALCULATION_INTERVAL
          value: "10s"
        - name: TRIP_STATUS_EVENTS
          value: "requested,matched,pickup,in_progress,completed,cancelled"
        - name: WEBSOCKET_MAX_CONNECTIONS
          value: "50000"
        - name: REAL_TIME_STREAMING
          value: "true"
        - name: LOCATION_SERVICE_ENDPOINT
          value: uber-location-service-{}.default.svc.cluster.local:80
        - name: ROUTE_SERVICE_ENDPOINT
          value: uber-route-service-{}.default.svc.cluster.local:8080
        - name: NOTIFICATION_SERVICE_ENDPOINT
          value: uber-notification-{}.default.svc.cluster.local:8080
        - name: TRIP_DB_HOST
          value: uber-trip-db-{}.default.svc.cluster.local:5432
        - name: REDIS_CACHE
          value: uber-redis-{}.default.svc.cluster.local:6379
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
"#, city, city, city, city, city, city, city, city, city);
    
    write_file(&format!("/tmp/uber-trip-tracking-{}.yaml", city), &tracking_config);
    exec(&format!("kubectl apply -f /tmp/uber-trip-tracking-{}.yaml", city));
}

fn deploy_safety_fraud_detection(cities: &[&str]) {
    echo("🛡️  Deploying Uber safety and fraud detection systems");
    
    for city in cities {
        // Deploy real-time fraud detection
        deploy_real_time_fraud_detection(city);
        
        // Deploy safety monitoring service
        deploy_safety_monitoring(city);
        
        // Deploy emergency response system
        deploy_emergency_response_system(city);
    }
}

fn deploy_real_time_fraud_detection(city: &str) {
    let fraud_config = format!(r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: uber-fraud-detection-{}
spec:
  replicas: 800
  selector:
    matchLabels:
      app: uber-fraud-detection
      city: {}
  template:
    metadata:
      labels:
        app: uber-fraud-detection
        city: {}
    spec:
      containers:
      - name: fraud-detector
        image: uber/fraud-detection:latest
        ports:
        - containerPort: 8080
        resources:
          requests:
            memory: "12Gi"
            cpu: "6000m"
            nvidia.com/gpu: 1
          limits:
            memory: "24Gi"
            cpu: "12000m"
            nvidia.com/gpu: 2
        env:
        - name: FRAUD_CITY
          value: "{}"
        - name: ML_MODEL_ENDPOINT
          value: uber-ml-fraud-{}.default.svc.cluster.local:8501
        - name: FEATURE_STORE_ENDPOINT
          value: uber-feature-store-{}.default.svc.cluster.local:8080
        - name: REAL_TIME_SCORING
          value: "true"
        - name: FRAUD_THRESHOLD_SCORE
          value: "0.85"
        - name: ANOMALY_DETECTION_MODELS
          value: "isolation_forest,one_class_svm,autoencoder"
        - name: BEHAVIOR_ANALYSIS_WINDOW
          value: "24h"
        - name: VELOCITY_CHECKS_ENABLED
          value: "true"
        - name: GEOLOCATION_VALIDATION
          value: "true"
        - name: PAYMENT_PATTERN_ANALYSIS
          value: "true"
        - name: ALERT_WEBHOOK
          value: "https://uber-security-alerts.internal.uber.com/webhook"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 60
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 15
"#, city, city, city, city, city, city);
    
    write_file(&format!("/tmp/uber-fraud-{}.yaml", city), &fraud_config);
    exec(&format!("kubectl apply -f /tmp/uber-fraud-{}.yaml", city));
}

fn deploy_ml_analytics_platform(cities: &[&str]) {
    echo("🤖 Deploying Uber ML analytics and optimization platform");
    
    for city in cities {
        // Deploy machine learning feature store
        deploy_ml_feature_store(city);
        
        // Deploy model serving infrastructure
        deploy_ml_model_serving(city);
        
        // Deploy real-time analytics pipeline
        deploy_realtime_analytics_pipeline(city);
    }
}

fn deploy_ml_model_serving(city: &str) {
    let ml_serving_config = format!(r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: uber-ml-serving-{}
spec:
  replicas: 1000
  selector:
    matchLabels:
      app: uber-ml-serving
      city: {}
  template:
    metadata:
      labels:
        app: uber-ml-serving
        city: {}
    spec:
      containers:
      - name: tensorflow-serving
        image: tensorflow/serving:latest-gpu
        ports:
        - containerPort: 8501
        - containerPort: 8500
        resources:
          requests:
            memory: "16Gi"
            cpu: "8000m"
            nvidia.com/gpu: 2
          limits:
            memory: "32Gi"
            cpu: "16000m"
            nvidia.com/gpu: 4
        env:
        - name: MODEL_BASE_PATH
          value: /models
        - name: MODEL_NAME
          value: uber_multitask_model
        - name: SERVING_CITY
          value: "{}"
        - name: TENSORFLOW_SERVING_BATCHING_PARAMETERS_FILE
          value: /config/batching_config.txt
        - name: TENSORFLOW_SERVING_MODEL_CONFIG_FILE
          value: /config/model_config.txt
        volumeMounts:
        - name: model-storage
          mountPath: /models
        - name: serving-config
          mountPath: /config
      volumes:
      - name: model-storage
        persistentVolumeClaim:
          claimName: uber-ml-models-{}
      - name: serving-config
        configMap:
          name: uber-ml-serving-config-{}
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: uber-ml-serving-config-{}
data:
  batching_config.txt: |
    max_batch_size {{ value: 128 }}
    batch_timeout_micros {{ value: 100000 }}
    max_enqueued_batches {{ value: 1000 }}
    num_batch_threads {{ value: 8 }}
  model_config.txt: |
    model_config_list {{
      config {{
        name: "eta_prediction"
        base_path: "/models/eta_prediction"
        model_platform: "tensorflow"
        model_version_policy {{
          latest {{
            num_versions: 2
          }}
        }}
      }}
      config {{
        name: "demand_forecasting"
        base_path: "/models/demand_forecasting"
        model_platform: "tensorflow"
        model_version_policy {{
          latest {{
            num_versions: 2
          }}
        }}
      }}
      config {{
        name: "fraud_detection"
        base_path: "/models/fraud_detection"
        model_platform: "tensorflow"
        model_version_policy {{
          latest {{
            num_versions: 2
          }}
        }}
      }}
    }}
"#, city, city, city, city, city, city, city);
    
    write_file(&format!("/tmp/uber-ml-serving-{}.yaml", city), &ml_serving_config);
    exec(&format!("kubectl apply -f /tmp/uber-ml-serving-{}.yaml", city));
}