// Amazon Database Solutions at enterprise scale with Rash
// Demonstrates RDS, DynamoDB, Aurora, and other database services

#[rash::main]
fn amazon_database_enterprise_solutions() {
    let regions = vec!["us-east-1", "us-west-2", "eu-west-1", "ap-southeast-1"];
    let read_replicas_per_region = "50";
    let dynamo_provisioned_reads = "1000000"; // 1M read capacity units
    let dynamo_provisioned_writes = "500000"; // 500K write capacity units
    
    echo("🗄️ Amazon Database Enterprise Solutions deployment");
    
    // Deploy Aurora Global Database clusters
    deploy_aurora_global_clusters(&regions);
    
    // Deploy DynamoDB Global Tables
    deploy_dynamodb_global_tables(&regions, &dynamo_provisioned_reads, &dynamo_provisioned_writes);
    
    // Deploy RDS Multi-AZ deployments
    deploy_rds_multi_az_clusters(&regions, &read_replicas_per_region);
    
    // Deploy ElastiCache Redis clusters
    deploy_elasticache_redis_clusters(&regions);
    
    // Deploy Amazon DocumentDB clusters
    deploy_documentdb_clusters(&regions);
    
    // Deploy Amazon Neptune graph databases
    deploy_neptune_graph_databases(&regions);
    
    // Setup database monitoring and performance insights
    deploy_database_monitoring(&regions);
    
    // Configure automated backups and disaster recovery
    configure_database_disaster_recovery(&regions);
    
    echo("✅ Amazon database enterprise solutions deployment completed");
}

fn deploy_aurora_global_clusters(regions: &[&str]) {
    echo("🌍 Deploying Aurora Global Database clusters");
    
    let primary_region = regions[0];
    let secondary_regions = &regions[1..];
    
    // Create primary Aurora cluster
    create_aurora_primary_cluster(primary_region);
    
    // Create Aurora global cluster
    create_aurora_global_cluster(primary_region);
    
    // Add secondary regions to global cluster
    for region in secondary_regions {
        add_aurora_secondary_cluster(region, primary_region);
    }
    
    // Configure Aurora Serverless v2 scaling
    configure_aurora_serverless_scaling(regions);
}

fn create_aurora_primary_cluster(region: &str) {
    let cluster_config = format!(r#"
aws rds create-db-cluster \
    --db-cluster-identifier amazon-aurora-primary-{} \
    --engine aurora-mysql \
    --engine-version 8.0.mysql_aurora.3.02.0 \
    --master-username amazonadmin \
    --master-user-password $(aws secretsmanager get-secret-value --secret-id amazon-db-master-password --query SecretString --output text) \
    --vpc-security-group-ids sg-amazon-aurora \
    --db-subnet-group-name amazon-aurora-subnet-group-{} \
    --backup-retention-period 35 \
    --preferred-backup-window "03:00-04:00" \
    --preferred-maintenance-window "sun:04:00-sun:05:00" \
    --enable-cloudwatch-logs-exports error general slowquery \
    --deletion-protection \
    --enable-iam-database-authentication \
    --enable-performance-insights \
    --performance-insights-retention-period 731 \
    --serverless-v2-scaling-configuration 'MinCapacity=0.5,MaxCapacity=128' \
    --tags 'Key=Name,Value=Amazon-Aurora-Primary-{}' 'Key=Environment,Value=Production' 'Key=Team,Value=Amazon-Database' \
    --region {}
"#, region, region, region, region);
    
    exec(&cluster_config);
    
    // Create cluster instances
    let instance_count = 10; // 10 instances per cluster for high availability
    for i in 1..=instance_count {
        let instance_config = format!(r#"
aws rds create-db-instance \
    --db-instance-identifier amazon-aurora-primary-{}-instance-{} \
    --db-instance-class db.r6g.4xlarge \
    --engine aurora-mysql \
    --db-cluster-identifier amazon-aurora-primary-{} \
    --performance-insights-enabled \
    --performance-insights-retention-period 731 \
    --monitoring-interval 60 \
    --monitoring-role-arn arn:aws:iam::123456789012:role/rds-monitoring-role \
    --enable-performance-insights \
    --tags 'Key=Name,Value=Amazon-Aurora-Primary-{}-Instance-{}' 'Key=InstanceNumber,Value={}' \
    --region {}
"#, region, i, region, region, i, i, region);
        
        exec(&instance_config);
    }
}

fn create_aurora_global_cluster(primary_region: &str) {
    let global_cluster_config = format!(r#"
aws rds create-global-cluster \
    --global-cluster-identifier amazon-aurora-global \
    --source-db-cluster-identifier amazon-aurora-primary-{} \
    --engine aurora-mysql \
    --engine-version 8.0.mysql_aurora.3.02.0 \
    --deletion-protection \
    --region {}
"#, primary_region, primary_region);
    
    exec(&global_cluster_config);
}

fn add_aurora_secondary_cluster(region: &str, primary_region: &str) {
    let secondary_cluster_config = format!(r#"
aws rds create-db-cluster \
    --db-cluster-identifier amazon-aurora-secondary-{} \
    --engine aurora-mysql \
    --global-cluster-identifier amazon-aurora-global \
    --vpc-security-group-ids sg-amazon-aurora \
    --db-subnet-group-name amazon-aurora-subnet-group-{} \
    --backup-retention-period 35 \
    --enable-cloudwatch-logs-exports error general slowquery \
    --deletion-protection \
    --enable-iam-database-authentication \
    --enable-performance-insights \
    --performance-insights-retention-period 731 \
    --serverless-v2-scaling-configuration 'MinCapacity=0.5,MaxCapacity=64' \
    --tags 'Key=Name,Value=Amazon-Aurora-Secondary-{}' 'Key=Environment,Value=Production' 'Key=Team,Value=Amazon-Database' \
    --region {}
"#, region, region, region, region);
    
    exec(&secondary_cluster_config);
    
    // Create read replica instances
    let replica_count = 5;
    for i in 1..=replica_count {
        let replica_config = format!(r#"
aws rds create-db-instance \
    --db-instance-identifier amazon-aurora-secondary-{}-instance-{} \
    --db-instance-class db.r6g.2xlarge \
    --engine aurora-mysql \
    --db-cluster-identifier amazon-aurora-secondary-{} \
    --performance-insights-enabled \
    --performance-insights-retention-period 731 \
    --monitoring-interval 60 \
    --monitoring-role-arn arn:aws:iam::123456789012:role/rds-monitoring-role \
    --tags 'Key=Name,Value=Amazon-Aurora-Secondary-{}-Instance-{}' 'Key=InstanceNumber,Value={}' \
    --region {}
"#, region, i, region, region, i, i, region);
        
        exec(&replica_config);
    }
}

fn deploy_dynamodb_global_tables(regions: &[&str], read_capacity: &str, write_capacity: &str) {
    echo("⚡ Deploying DynamoDB Global Tables");
    
    let tables = vec![
        ("amazon-users", "user_id", "S", "email", "S"),
        ("amazon-products", "product_id", "S", "category", "S"),
        ("amazon-orders", "order_id", "S", "customer_id", "S"),
        ("amazon-inventory", "sku", "S", "warehouse_id", "S"),
        ("amazon-recommendations", "user_id", "S", "timestamp", "N"),
        ("amazon-sessions", "session_id", "S", "user_id", "S"),
        ("amazon-payments", "payment_id", "S", "order_id", "S"),
        ("amazon-reviews", "review_id", "S", "product_id", "S"),
    ];
    
    let primary_region = regions[0];
    
    // Create tables in primary region first
    for (table_name, hash_key, hash_type, range_key, range_type) in &tables {
        create_dynamodb_table(primary_region, table_name, hash_key, hash_type, range_key, range_type, read_capacity, write_capacity);
    }
    
    // Wait for tables to be active
    for (table_name, _, _, _, _) in &tables {
        exec(&format!("aws dynamodb wait table-exists --table-name {} --region {}", table_name, primary_region));
    }
    
    // Create global tables
    for (table_name, _, _, _, _) in &tables {
        create_global_table(table_name, regions);
    }
}

fn create_dynamodb_table(region: &str, table_name: &str, hash_key: &str, hash_type: &str, range_key: &str, range_type: &str, read_capacity: &str, write_capacity: &str) {
    let table_config = format!(r#"
aws dynamodb create-table \
    --table-name {} \
    --attribute-definitions \
        AttributeName={},AttributeType={} \
        AttributeName={},AttributeType={} \
    --key-schema \
        AttributeName={},KeyType=HASH \
        AttributeName={},KeyType=RANGE \
    --billing-mode PROVISIONED \
    --provisioned-throughput ReadCapacityUnits={},WriteCapacityUnits={} \
    --global-secondary-indexes \
        'IndexName={}-gsi-1,KeySchema=[{{AttributeName={},KeyType=HASH}}],Projection={{ProjectionType=ALL}},ProvisionedThroughput={{ReadCapacityUnits=50000,WriteCapacityUnits=25000}}' \
    --stream-specification StreamEnabled=true,StreamViewType=NEW_AND_OLD_IMAGES \
    --sse-specification Enabled=true,SSEType=KMS,KMSMasterKeyId=arn:aws:kms:{}:123456789012:key/amazon-dynamodb-key \
    --point-in-time-recovery-specification PointInTimeRecoveryEnabled=true \
    --tags 'Key=Name,Value={}' 'Key=Environment,Value=Production' 'Key=Team,Value=Amazon-Database' \
    --region {}
"#, table_name, hash_key, hash_type, range_key, range_type, hash_key, range_key, read_capacity, write_capacity, table_name, range_key, region, table_name, region);
    
    exec(&table_config);
    
    // Enable auto scaling
    enable_dynamodb_autoscaling(region, table_name);
}

fn enable_dynamodb_autoscaling(region: &str, table_name: &str) {
    // Enable auto scaling for read capacity
    let read_scaling_config = format!(r#"
aws application-autoscaling register-scalable-target \
    --service-namespace dynamodb \
    --resource-id table/{} \
    --scalable-dimension dynamodb:table:ReadCapacityUnits \
    --min-capacity 1000 \
    --max-capacity 2000000 \
    --region {}
"#, table_name, region);
    
    exec(&read_scaling_config);
    
    // Enable auto scaling for write capacity
    let write_scaling_config = format!(r#"
aws application-autoscaling register-scalable-target \
    --service-namespace dynamodb \
    --resource-id table/{} \
    --scalable-dimension dynamodb:table:WriteCapacityUnits \
    --min-capacity 500 \
    --max-capacity 1000000 \
    --region {}
"#, table_name, region);
    
    exec(&write_scaling_config);
    
    // Create scaling policies
    let read_policy_config = format!(r#"
aws application-autoscaling put-scaling-policy \
    --service-namespace dynamodb \
    --resource-id table/{} \
    --scalable-dimension dynamodb:table:ReadCapacityUnits \
    --policy-name {}-read-scaling-policy \
    --policy-type TargetTrackingScaling \
    --target-tracking-scaling-policy-configuration '{{
        "TargetValue": 70.0,
        "PredefinedMetricSpecification": {{
            "PredefinedMetricType": "DynamoDBReadCapacityUtilization"
        }},
        "ScaleOutCooldown": 60,
        "ScaleInCooldown": 60
    }}' \
    --region {}
"#, table_name, table_name, region);
    
    exec(&read_policy_config);
    
    let write_policy_config = format!(r#"
aws application-autoscaling put-scaling-policy \
    --service-namespace dynamodb \
    --resource-id table/{} \
    --scalable-dimension dynamodb:table:WriteCapacityUnits \
    --policy-name {}-write-scaling-policy \
    --policy-type TargetTrackingScaling \
    --target-tracking-scaling-policy-configuration '{{
        "TargetValue": 70.0,
        "PredefinedMetricSpecification": {{
            "PredefinedMetricType": "DynamoDBWriteCapacityUtilization"
        }},
        "ScaleOutCooldown": 60,
        "ScaleInCooldown": 60
    }}' \
    --region {}
"#, table_name, table_name, region);
    
    exec(&write_policy_config);
}

fn create_global_table(table_name: &str, regions: &[&str]) {
    let mut region_list = String::new();
    for (i, region) in regions.iter().enumerate() {
        if i > 0 {
            region_list.push(',');
        }
        region_list.push_str(&format!("{{RegionName={}}}", region));
    }
    
    let global_table_config = format!(r#"
aws dynamodb create-global-table \
    --global-table-name {} \
    --replication-group [{}] \
    --region {}
"#, table_name, region_list, regions[0]);
    
    exec(&global_table_config);
}

fn deploy_elasticache_redis_clusters(regions: &[&str]) {
    echo("🚀 Deploying ElastiCache Redis clusters");
    
    for region in regions {
        // Create Redis replication groups
        create_redis_replication_groups(region);
        
        // Create Redis Global Datastore
        create_redis_global_datastore(region);
        
        // Configure Redis cluster mode
        configure_redis_cluster_mode(region);
    }
}

fn create_redis_replication_groups(region: &str) {
    let redis_clusters = vec![
        ("amazon-session-cache", "cache.r6g.2xlarge", "Session storage for Amazon web applications"),
        ("amazon-product-cache", "cache.r6g.4xlarge", "Product catalog caching"),
        ("amazon-recommendation-cache", "cache.r6g.8xlarge", "ML recommendation caching"),
        ("amazon-api-cache", "cache.r6g.xlarge", "API response caching"),
        ("amazon-search-cache", "cache.r6g.2xlarge", "Search results caching"),
    ];
    
    for (cluster_name, node_type, description) in redis_clusters {
        let redis_config = format!(r#"
aws elasticache create-replication-group \
    --replication-group-id {}-{} \
    --replication-group-description "{}" \
    --num-cache-clusters 6 \
    --cache-node-type {} \
    --engine redis \
    --engine-version 7.0 \
    --port 6379 \
    --parameter-group-name default.redis7 \
    --subnet-group-name amazon-cache-subnet-group-{} \
    --security-group-ids sg-amazon-redis \
    --at-rest-encryption-enabled \
    --transit-encryption-enabled \
    --auth-token $(aws secretsmanager get-secret-value --secret-id amazon-redis-auth-token --query SecretString --output text) \
    --automatic-failover-enabled \
    --multi-az-enabled \
    --snapshot-retention-limit 7 \
    --snapshot-window "03:00-05:00" \
    --preferred-maintenance-window "sun:05:00-sun:07:00" \
    --notification-topic-arn arn:aws:sns:{}:123456789012:amazon-elasticache-notifications \
    --log-delivery-configurations 'LogType=slow-log,DestinationType=cloudwatch-logs,LogFormat=json,Enabled=true,DestinationDetails={{CloudWatchLogsDetails={{LogGroup=amazon-redis-slow-logs}}}}' \
    --tags 'Key=Name,Value={}-{}' 'Key=Environment,Value=Production' 'Key=Team,Value=Amazon-Database' \
    --region {}
"#, cluster_name, region, description, node_type, region, region, cluster_name, region, region);
        
        exec(&redis_config);
    }
}

fn deploy_rds_multi_az_clusters(regions: &[&str], read_replicas: &str) {
    echo("🏢 Deploying RDS Multi-AZ clusters");
    
    for region in regions {
        // Deploy PostgreSQL clusters
        deploy_postgresql_clusters(region, read_replicas);
        
        // Deploy MySQL clusters
        deploy_mysql_clusters(region, read_replicas);
        
        // Deploy Oracle Enterprise clusters
        deploy_oracle_clusters(region, read_replicas);
        
        // Deploy SQL Server clusters
        deploy_sqlserver_clusters(region, read_replicas);
    }
}

fn deploy_postgresql_clusters(region: &str, read_replicas: &str) {
    let postgresql_instances = vec![
        ("amazon-main-postgresql", "db.r6g.8xlarge", "Main PostgreSQL database for Amazon services"),
        ("amazon-analytics-postgresql", "db.r6g.4xlarge", "Analytics and reporting database"),
        ("amazon-warehouse-postgresql", "db.r6g.2xlarge", "Data warehouse for business intelligence"),
    ];
    
    for (instance_name, instance_class, description) in postgresql_instances {
        let postgresql_config = format!(r#"
aws rds create-db-instance \
    --db-instance-identifier {}-{} \
    --db-instance-class {} \
    --engine postgres \
    --engine-version 15.4 \
    --master-username amazonadmin \
    --master-user-password $(aws secretsmanager get-secret-value --secret-id amazon-postgresql-master-password --query SecretString --output text) \
    --allocated-storage 10000 \
    --max-allocated-storage 100000 \
    --storage-type gp3 \
    --storage-encrypted \
    --kms-key-id arn:aws:kms:{}:123456789012:key/amazon-rds-key \
    --vpc-security-group-ids sg-amazon-postgresql \
    --db-subnet-group-name amazon-postgresql-subnet-group-{} \
    --multi-az \
    --backup-retention-period 35 \
    --preferred-backup-window "03:00-04:00" \
    --preferred-maintenance-window "sun:04:00-sun:05:00" \
    --enable-cloudwatch-logs-exports postgresql \
    --deletion-protection \
    --enable-iam-database-authentication \
    --enable-performance-insights \
    --performance-insights-retention-period 731 \
    --monitoring-interval 60 \
    --monitoring-role-arn arn:aws:iam::123456789012:role/rds-monitoring-role \
    --tags 'Key=Name,Value={}-{}' 'Key=Environment,Value=Production' 'Key=Team,Value=Amazon-Database' \
    --region {}
"#, instance_name, region, instance_class, region, region, instance_name, region, region);
        
        exec(&postgresql_config);
        
        // Create read replicas
        for i in 1..=read_replicas.parse::<i32>().unwrap_or(10).min(10) {
            let replica_config = format!(r#"
aws rds create-db-instance-read-replica \
    --db-instance-identifier {}-{}-replica-{} \
    --source-db-instance-identifier {}-{} \
    --db-instance-class db.r6g.2xlarge \
    --auto-minor-version-upgrade \
    --enable-performance-insights \
    --performance-insights-retention-period 731 \
    --monitoring-interval 60 \
    --monitoring-role-arn arn:aws:iam::123456789012:role/rds-monitoring-role \
    --tags 'Key=Name,Value={}-{}-Replica-{}' 'Key=ReplicaNumber,Value={}' \
    --region {}
"#, instance_name, region, i, instance_name, region, instance_name, region, i, i, region);
            
            exec(&replica_config);
        }
    }
}

fn deploy_documentdb_clusters(regions: &[&str]) {
    echo("📄 Deploying Amazon DocumentDB clusters");
    
    for region in regions {
        let docdb_config = format!(r#"
aws docdb create-db-cluster \
    --db-cluster-identifier amazon-documentdb-{} \
    --engine docdb \
    --engine-version 5.0.0 \
    --master-username amazonadmin \
    --master-user-password $(aws secretsmanager get-secret-value --secret-id amazon-docdb-master-password --query SecretString --output text) \
    --vpc-security-group-ids sg-amazon-documentdb \
    --db-subnet-group-name amazon-documentdb-subnet-group-{} \
    --backup-retention-period 35 \
    --preferred-backup-window "03:00-04:00" \
    --preferred-maintenance-window "sun:04:00-sun:05:00" \
    --enable-cloudwatch-logs-exports audit profiler \
    --deletion-protection \
    --storage-encrypted \
    --kms-key-id arn:aws:kms:{}:123456789012:key/amazon-documentdb-key \
    --tags 'Key=Name,Value=Amazon-DocumentDB-{}' 'Key=Environment,Value=Production' 'Key=Team,Value=Amazon-Database' \
    --region {}
"#, region, region, region, region, region);
        
        exec(&docdb_config);
        
        // Create DocumentDB instances
        let instance_count = 6;
        for i in 1..=instance_count {
            let instance_config = format!(r#"
aws docdb create-db-instance \
    --db-instance-identifier amazon-documentdb-{}-instance-{} \
    --db-instance-class db.r6g.2xlarge \
    --engine docdb \
    --db-cluster-identifier amazon-documentdb-{} \
    --enable-performance-insights \
    --performance-insights-kms-key-id arn:aws:kms:{}:123456789012:key/amazon-documentdb-key \
    --tags 'Key=Name,Value=Amazon-DocumentDB-{}-Instance-{}' 'Key=InstanceNumber,Value={}' \
    --region {}
"#, region, i, region, region, region, i, i, region);
            
            exec(&instance_config);
        }
    }
}

fn deploy_neptune_graph_databases(regions: &[&str]) {
    echo("🕸️ Deploying Amazon Neptune graph databases");
    
    for region in regions {
        let neptune_config = format!(r#"
aws neptune create-db-cluster \
    --db-cluster-identifier amazon-neptune-{} \
    --engine neptune \
    --engine-version 1.2.1.0 \
    --vpc-security-group-ids sg-amazon-neptune \
    --db-subnet-group-name amazon-neptune-subnet-group-{} \
    --backup-retention-period 35 \
    --preferred-backup-window "03:00-04:00" \
    --preferred-maintenance-window "sun:04:00-sun:05:00" \
    --enable-cloudwatch-logs-exports audit \
    --deletion-protection \
    --storage-encrypted \
    --kms-key-id arn:aws:kms:{}:123456789012:key/amazon-neptune-key \
    --enable-iam-database-authentication \
    --tags 'Key=Name,Value=Amazon-Neptune-{}' 'Key=Environment,Value=Production' 'Key=Team,Value=Amazon-Database' \
    --region {}
"#, region, region, region, region, region);
        
        exec(&neptune_config);
        
        // Create Neptune instances
        let instance_count = 4;
        for i in 1..=instance_count {
            let instance_config = format!(r#"
aws neptune create-db-instance \
    --db-instance-identifier amazon-neptune-{}-instance-{} \
    --db-instance-class db.r6g.2xlarge \
    --engine neptune \
    --db-cluster-identifier amazon-neptune-{} \
    --tags 'Key=Name,Value=Amazon-Neptune-{}-Instance-{}' 'Key=InstanceNumber,Value={}' \
    --region {}
"#, region, i, region, region, i, i, region);
            
            exec(&instance_config);
        }
    }
}

fn deploy_database_monitoring(regions: &[&str]) {
    echo("📊 Deploying database monitoring and performance insights");
    
    for region in regions {
        // Create CloudWatch dashboard for databases
        create_database_dashboard(region);
        
        // Setup database alarms
        setup_database_alarms(region);
        
        // Configure Performance Insights
        configure_performance_insights(region);
    }
}

fn create_database_dashboard(region: &str) {
    let dashboard_config = format!(r#"
{{
  "widgets": [
    {{
      "type": "metric",
      "properties": {{
        "metrics": [
          ["AWS/RDS", "CPUUtilization", "DBInstanceIdentifier", "amazon-main-postgresql-{}"],
          ["AWS/RDS", "DatabaseConnections", "DBInstanceIdentifier", "amazon-main-postgresql-{}"],
          ["AWS/RDS", "ReadLatency", "DBInstanceIdentifier", "amazon-main-postgresql-{}"],
          ["AWS/RDS", "WriteLatency", "DBInstanceIdentifier", "amazon-main-postgresql-{}"],
          ["AWS/DynamoDB", "ConsumedReadCapacityUnits", "TableName", "amazon-users"],
          ["AWS/DynamoDB", "ConsumedWriteCapacityUnits", "TableName", "amazon-users"],
          ["AWS/ElastiCache", "CPUUtilization", "CacheClusterId", "amazon-session-cache-{}-001"],
          ["AWS/ElastiCache", "CurrConnections", "CacheClusterId", "amazon-session-cache-{}-001"]
        ],
        "period": 300,
        "stat": "Average",
        "region": "{}",
        "title": "Amazon Database Performance - {}"
      }}
    }},
    {{
      "type": "log",
      "properties": {{
        "query": "SOURCE '/aws/rds/instance/amazon-main-postgresql-{}/postgresql' | fields @timestamp, @message | filter @message like /ERROR/ | sort @timestamp desc | limit 100",
        "region": "{}",
        "title": "Database Errors - {}",
        "view": "table"
      }}
    }}
  ]
}}
"#, region, region, region, region, region, region, region, region, region, region, region);
    
    write_file(&format!("/tmp/db-dashboard-{}.json", region), &dashboard_config);
    exec(&format!("aws cloudwatch put-dashboard --dashboard-name Amazon-Database-Dashboard-{} --dashboard-body file:///tmp/db-dashboard-{}.json --region {}", region, region, region));
}

fn configure_database_disaster_recovery(regions: &[&str]) {
    echo("🔄 Configuring database disaster recovery");
    
    for region in regions {
        // Configure automated backups
        configure_automated_backups(region);
        
        // Setup cross-region backup replication
        setup_cross_region_backup_replication(region);
        
        // Configure point-in-time recovery
        configure_point_in_time_recovery(region);
    }
}

fn configure_automated_backups(region: &str) {
    // Create backup vault
    let backup_vault_config = format!(r#"
aws backup create-backup-vault \
    --backup-vault-name amazon-database-backup-vault-{} \
    --encryption-key-id arn:aws:kms:{}:123456789012:key/amazon-backup-key \
    --backup-vault-tags 'Environment=Production,Team=Amazon-Database,Region={}' \
    --region {}
"#, region, region, region, region);
    
    exec(&backup_vault_config);
    
    // Create backup plan
    let backup_plan_config = format!(r#"
aws backup create-backup-plan \
    --backup-plan '{{
        "BackupPlanName": "Amazon-Database-Backup-Plan-{}",
        "Rules": [
            {{
                "RuleName": "DailyBackup",
                "TargetBackupVault": "amazon-database-backup-vault-{}",
                "ScheduleExpression": "cron(0 5 ? * * *)",
                "StartWindowMinutes": 480,
                "CompletionWindowMinutes": 10080,
                "Lifecycle": {{
                    "MoveToColdStorageAfterDays": 30,
                    "DeleteAfterDays": 365
                }},
                "RecoveryPointTags": {{
                    "BackupType": "Daily",
                    "Environment": "Production"
                }}
            }},
            {{
                "RuleName": "WeeklyBackup", 
                "TargetBackupVault": "amazon-database-backup-vault-{}",
                "ScheduleExpression": "cron(0 2 ? * SUN *)",
                "StartWindowMinutes": 480,
                "CompletionWindowMinutes": 10080,
                "Lifecycle": {{
                    "MoveToColdStorageAfterDays": 7,
                    "DeleteAfterDays": 2555
                }},
                "RecoveryPointTags": {{
                    "BackupType": "Weekly",
                    "Environment": "Production"
                }}
            }}
        ]
    }}' \
    --backup-plan-tags 'Environment=Production,Team=Amazon-Database' \
    --region {}
"#, region, region, region, region);
    
    exec(&backup_plan_config);
}