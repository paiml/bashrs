// Amazon AWS Global Infrastructure deployment with Rash
// Demonstrates enterprise-scale AWS deployment across multiple regions

#[rash::main]
fn amazon_aws_global_deployment() {
    let account_id = "123456789012"; // Amazon production account
    let primary_region = "us-east-1";
    let secondary_regions = vec!["us-west-2", "eu-west-1", "ap-southeast-1"];
    
    echo("☁️  Amazon AWS Global Infrastructure deployment");
    
    // Install and configure AWS CLI v2
    install_aws_cli_v2();
    configure_aws_credentials(&account_id);
    
    // Deploy global VPC infrastructure
    deploy_global_vpc_infrastructure(&primary_region, &secondary_regions);
    
    // Setup Amazon EKS clusters across regions
    deploy_multi_region_eks(&primary_region, &secondary_regions);
    
    // Deploy Amazon's core services
    deploy_amazon_core_services(&primary_region);
    
    // Setup global load balancing with Route 53
    configure_route53_global_lb(&primary_region, &secondary_regions);
    
    // Deploy Lambda functions for serverless compute
    deploy_lambda_functions(&primary_region, &secondary_regions);
    
    // Setup CloudWatch monitoring and alerting
    configure_cloudwatch_enterprise(&account_id, &primary_region);
    
    // Configure AWS Security Hub for compliance
    setup_aws_security_hub(&account_id, &secondary_regions);
    
    // Deploy additional Amazon ecosystems
    deploy_amazon_prime_infrastructure(&secondary_regions);
    deploy_amazon_retail_ecosystem(&secondary_regions);
    deploy_amazon_alexa_ecosystem(&secondary_regions);
    setup_amazon_enterprise_monitoring(&secondary_regions);
    
    echo("✅ Amazon AWS global infrastructure deployment completed");
}

fn install_aws_cli_v2() {
    let aws_cli_url = "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip";
    
    curl(aws_cli_url, "/tmp/awscliv2.zip");
    cd("/tmp");
    exec("unzip -q awscliv2.zip");
    exec("sudo ./aws/install --update");
    
    // Verify installation
    exec("aws --version");
}

fn configure_aws_credentials(account_id: &str) {
    // Configure AWS credentials using IAM roles for production
    let role_arn = format!("arn:aws:iam::{}:role/AmazonProductionDeploymentRole", account_id);
    
    exec(&format!("aws sts assume-role --role-arn {} --role-session-name amazon-deployment", role_arn));
    
    // Set default region and output format
    exec("aws configure set default.region us-east-1");
    exec("aws configure set default.output json");
}

fn deploy_global_vpc_infrastructure(primary: &str, regions: &[&str]) {
    // Deploy primary VPC with full infrastructure
    deploy_primary_vpc(primary);
    
    // Deploy secondary VPCs in other regions
    for region in regions {
        deploy_secondary_vpc(region);
        setup_vpc_peering(primary, region);
    }
    
    // Setup Transit Gateway for inter-region connectivity
    setup_transit_gateway(primary, regions);
}

fn deploy_primary_vpc(region: &str) {
    let vpc_config = format!(r#"
aws ec2 create-vpc \
    --cidr-block 10.0.0.0/16 \
    --tag-specifications 'ResourceType=vpc,Tags=[{{Key=Name,Value=Amazon-Primary-VPC}},{{Key=Environment,Value=Production}},{{Key=Team,Value=Amazon-Infrastructure}}]' \
    --region {}
"#, region);
    
    exec(&vpc_config);
    
    let vpc_id = exec_output("aws ec2 describe-vpcs --filters 'Name=tag:Name,Values=Amazon-Primary-VPC' --query 'Vpcs[0].VpcId' --output text");
    
    // Create subnets across availability zones
    let azs = vec!["a", "b", "c"];
    let mut subnet_id = 1;
    
    for az in azs {
        // Public subnet
        exec(&format!(r#"
aws ec2 create-subnet \
    --vpc-id {} \
    --cidr-block 10.0.{}.0/24 \
    --availability-zone {}{} \
    --tag-specifications 'ResourceType=subnet,Tags=[{{Key=Name,Value=Amazon-Public-Subnet-{}}},{{Key=Type,Value=Public}}]' \
    --region {}
"#, vpc_id, subnet_id, region, az, az, region));
        
        subnet_id += 1;
        
        // Private subnet
        exec(&format!(r#"
aws ec2 create-subnet \
    --vpc-id {} \
    --cidr-block 10.0.{}.0/24 \
    --availability-zone {}{} \
    --tag-specifications 'ResourceType=subnet,Tags=[{{Key=Name,Value=Amazon-Private-Subnet-{}}},{{Key=Type,Value=Private}}]' \
    --region {}
"#, vpc_id, subnet_id, region, az, az, region));
        
        subnet_id += 1;
    }
    
    // Create Internet Gateway
    exec(&format!(r#"
aws ec2 create-internet-gateway \
    --tag-specifications 'ResourceType=internet-gateway,Tags=[{{Key=Name,Value=Amazon-IGW}}]' \
    --region {}
"#, region));
    
    let igw_id = exec_output("aws ec2 describe-internet-gateways --filters 'Name=tag:Name,Values=Amazon-IGW' --query 'InternetGateways[0].InternetGatewayId' --output text");
    
    // Attach Internet Gateway to VPC
    exec(&format!("aws ec2 attach-internet-gateway --vpc-id {} --internet-gateway-id {} --region {}", vpc_id, igw_id, region));
}

fn deploy_secondary_vpc(region: &str) {
    let vpc_config = format!(r#"
aws ec2 create-vpc \
    --cidr-block 10.{}.0.0/16 \
    --tag-specifications 'ResourceType=vpc,Tags=[{{Key=Name,Value=Amazon-Secondary-VPC-{}}},{{Key=Environment,Value=Production}},{{Key=Team,Value=Amazon-Infrastructure}}]' \
    --region {}
"#, get_region_number(region), region, region);
    
    exec(&vpc_config);
}

fn setup_vpc_peering(primary_region: &str, secondary_region: &str) {
    let primary_vpc_id = exec_output(&format!(
        "aws ec2 describe-vpcs --filters 'Name=tag:Name,Values=Amazon-Primary-VPC' --query 'Vpcs[0].VpcId' --output text --region {}",
        primary_region
    ));
    
    let secondary_vpc_id = exec_output(&format!(
        "aws ec2 describe-vpcs --filters 'Name=tag:Name,Values=Amazon-Secondary-VPC-{}' --query 'Vpcs[0].VpcId' --output text --region {}",
        secondary_region, secondary_region
    ));
    
    // Create VPC peering connection
    exec(&format!(r#"
aws ec2 create-vpc-peering-connection \
    --vpc-id {} \
    --peer-vpc-id {} \
    --peer-region {} \
    --tag-specifications 'ResourceType=vpc-peering-connection,Tags=[{{Key=Name,Value=Amazon-VPC-Peering-{}-{}}}]' \
    --region {}
"#, primary_vpc_id, secondary_vpc_id, secondary_region, primary_region, secondary_region, primary_region));
}

fn setup_transit_gateway(primary_region: &str, regions: &[&str]) {
    // Create Transit Gateway in primary region
    exec(&format!(r#"
aws ec2 create-transit-gateway \
    --description "Amazon Global Transit Gateway" \
    --options DefaultRouteTableAssociation=enable,DefaultRouteTablePropagation=enable \
    --tag-specifications 'ResourceType=transit-gateway,Tags=[{{Key=Name,Value=Amazon-Global-TGW}}]' \
    --region {}
"#, primary_region));
    
    // Setup cross-region peering for other regions
    for region in regions {
        setup_tgw_peering(primary_region, region);
    }
}

fn setup_tgw_peering(primary: &str, secondary: &str) {
    let primary_tgw_id = exec_output(&format!(
        "aws ec2 describe-transit-gateways --filters 'Name=tag:Name,Values=Amazon-Global-TGW' --query 'TransitGateways[0].TransitGatewayId' --output text --region {}",
        primary
    ));
    
    // Create TGW in secondary region
    exec(&format!(r#"
aws ec2 create-transit-gateway \
    --description "Amazon Regional Transit Gateway {}" \
    --tag-specifications 'ResourceType=transit-gateway,Tags=[{{Key=Name,Value=Amazon-Regional-TGW-{}}}]' \
    --region {}
"#, secondary, secondary, secondary));
    
    let secondary_tgw_id = exec_output(&format!(
        "aws ec2 describe-transit-gateways --filters 'Name=tag:Name,Values=Amazon-Regional-TGW-{}' --query 'TransitGateways[0].TransitGatewayId' --output text --region {}",
        secondary, secondary
    ));
    
    // Create peering attachment
    exec(&format!(r#"
aws ec2 create-transit-gateway-peering-attachment \
    --transit-gateway-id {} \
    --peer-transit-gateway-id {} \
    --peer-region {} \
    --tag-specifications 'ResourceType=transit-gateway-attachment,Tags=[{{Key=Name,Value=Amazon-TGW-Peering-{}}}]' \
    --region {}
"#, primary_tgw_id, secondary_tgw_id, secondary, secondary, primary));
}

fn deploy_multi_region_eks(primary: &str, regions: &[&str]) {
    // Deploy primary EKS cluster
    deploy_eks_cluster(primary, "amazon-primary-eks", "2000");
    
    // Deploy regional EKS clusters
    for region in regions {
        deploy_eks_cluster(region, &format!("amazon-{}-eks", region), "1000");
    }
}

fn deploy_eks_cluster(region: &str, cluster_name: &str, node_count: &str) {
    let cluster_config = format!(r#"
aws eks create-cluster \
    --name {} \
    --version 1.28 \
    --role-arn arn:aws:iam::123456789012:role/AmazonEKSClusterServiceRole \
    --resources-vpc-config subnetIds=subnet-12345,subnet-67890,subnet-abcdef \
    --tags Environment=Production,Team=Amazon-EKS,Region={} \
    --region {}
"#, cluster_name, region, region);
    
    exec(&cluster_config);
    
    // Wait for cluster to be active
    exec(&format!("aws eks wait cluster-active --name {} --region {}", cluster_name, region));
    
    // Create node group
    exec(&format!(r#"
aws eks create-nodegroup \
    --cluster-name {} \
    --nodegroup-name {}-nodes \
    --node-role arn:aws:iam::123456789012:role/AmazonEKSNodeInstanceRole \
    --subnets subnet-12345 subnet-67890 \
    --instance-types m5.2xlarge \
    --scaling-config minSize=100,maxSize={},desiredSize={} \
    --disk-size 100 \
    --ami-type AL2_x86_64 \
    --capacity-type ON_DEMAND \
    --tags Environment=Production,Team=Amazon-EKS \
    --region {}
"#, cluster_name, cluster_name, node_count, node_count, region));
}

fn deploy_amazon_core_services(region: &str) {
    let services = vec![
        "amazon-retail-api",
        "amazon-prime-streaming", 
        "amazon-alexa-backend",
        "amazon-aws-console",
        "amazon-kindle-service",
        "amazon-logistics-tracker",
    ];
    
    for service in services {
        deploy_ecs_service(region, service);
    }
}

fn deploy_ecs_service(region: &str, service_name: &str) {
    // Create ECS cluster
    exec(&format!("aws ecs create-cluster --cluster-name amazon-{}-cluster --region {}", service_name, region));
    
    // Create task definition
    let task_def = format!(r#"
{{
  "family": "{}",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "2048",
  "memory": "4096",
  "executionRoleArn": "arn:aws:iam::123456789012:role/ecsTaskExecutionRole",
  "containerDefinitions": [
    {{
      "name": "{}",
      "image": "123456789012.dkr.ecr.{}.amazonaws.com/{}:latest",
      "portMappings": [
        {{
          "containerPort": 8080,
          "protocol": "tcp"
        }}
      ],
      "logConfiguration": {{
        "logDriver": "awslogs",
        "options": {{
          "awslogs-group": "/ecs/{}",
          "awslogs-region": "{}",
          "awslogs-stream-prefix": "ecs"
        }}
      }}
    }}
  ]
}}
"#, service_name, service_name, region, service_name, service_name, region);
    
    write_file(&format!("/tmp/{}-task-def.json", service_name), &task_def);
    exec(&format!("aws ecs register-task-definition --cli-input-json file:///tmp/{}-task-def.json --region {}", service_name, region));
    
    // Create ECS service
    exec(&format!(r#"
aws ecs create-service \
    --cluster amazon-{}-cluster \
    --service-name {}-service \
    --task-definition {} \
    --desired-count 100 \
    --launch-type FARGATE \
    --network-configuration "awsvpcConfiguration={{subnets=[subnet-12345,subnet-67890],securityGroups=[sg-12345],assignPublicIp=ENABLED}}" \
    --region {}
"#, service_name, service_name, service_name, region));
}

fn configure_route53_global_lb(primary: &str, regions: &[&str]) {
    // Create Route 53 hosted zone
    exec(r#"
aws route53 create-hosted-zone \
    --name amazon.com \
    --caller-reference $(date +%s) \
    --hosted-zone-config Comment="Amazon Global DNS",PrivateZone=false
"#);
    
    let hosted_zone_id = exec_output("aws route53 list-hosted-zones --query 'HostedZones[?Name==`amazon.com.`].Id' --output text");
    
    // Create global load balancer records
    for region in regions {
        create_route53_record(&hosted_zone_id, region);
    }
}

fn create_route53_record(zone_id: &str, region: &str) {
    let record_config = format!(r#"
{{
  "Changes": [
    {{
      "Action": "CREATE",
      "ResourceRecordSet": {{
        "Name": "{}.amazon.com",
        "Type": "A",
        "SetIdentifier": "{}",
        "Failover": "PRIMARY",
        "AliasTarget": {{
          "DNSName": "amazon-{}-lb.elb.{}.amazonaws.com",
          "EvaluateTargetHealth": true,
          "HostedZoneId": "Z35SXDOTRQ7X7K"
        }}
      }}
    }}
  ]
}}
"#, region, region, region, region);
    
    write_file(&format!("/tmp/route53-{}.json", region), &record_config);
    exec(&format!("aws route53 change-resource-record-sets --hosted-zone-id {} --change-batch file:///tmp/route53-{}.json", zone_id, region));
}

fn deploy_lambda_functions(primary: &str, regions: &[&str]) {
    let functions = vec![
        "amazon-order-processor",
        "amazon-recommendation-engine",
        "amazon-inventory-updater", 
        "amazon-payment-processor",
        "amazon-fraud-detector",
    ];
    
    for function in functions {
        deploy_lambda_function(primary, function);
        
        // Replicate to other regions
        for region in regions {
            replicate_lambda_function(primary, region, function);
        }
    }
}

fn deploy_lambda_function(region: &str, function_name: &str) {
    // Create IAM role for Lambda
    exec(&format!(r#"
aws iam create-role \
    --role-name {}-lambda-role \
    --assume-role-policy-document '{{"Version":"2012-10-17","Statement":[{{"Effect":"Allow","Principal":{{"Service":"lambda.amazonaws.com"}},"Action":"sts:AssumeRole"}}]}}'
"#, function_name));
    
    // Attach basic execution policy
    exec(&format!("aws iam attach-role-policy --role-name {}-lambda-role --policy-arn arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole", function_name));
    
    // Create deployment package (placeholder)
    echo(&format!("console.log('Amazon {} function');", function_name)) > &format!("/tmp/{}.js", function_name);
    cd("/tmp");
    exec(&format!("zip {}.zip {}.js", function_name, function_name));
    
    // Create Lambda function
    exec(&format!(r#"
aws lambda create-function \
    --function-name {} \
    --runtime nodejs18.x \
    --role arn:aws:iam::123456789012:role/{}-lambda-role \
    --handler index.handler \
    --zip-file fileb://{}.zip \
    --timeout 300 \
    --memory-size 1024 \
    --tags Environment=Production,Team=Amazon-Lambda \
    --region {}
"#, function_name, function_name, function_name, region));
}

fn replicate_lambda_function(source_region: &str, target_region: &str, function_name: &str) {
    // Get function configuration from source region
    exec(&format!(r#"
aws lambda get-function \
    --function-name {} \
    --region {} \
    --query 'Code.Location' \
    --output text | xargs curl -o /tmp/{}-{}.zip
"#, function_name, source_region, function_name, target_region));
    
    // Create function in target region
    exec(&format!(r#"
aws lambda create-function \
    --function-name {} \
    --runtime nodejs18.x \
    --role arn:aws:iam::123456789012:role/{}-lambda-role \
    --handler index.handler \
    --zip-file fileb:///tmp/{}-{}.zip \
    --timeout 300 \
    --memory-size 1024 \
    --tags Environment=Production,Team=Amazon-Lambda \
    --region {}
"#, function_name, function_name, function_name, target_region, target_region));
}

fn configure_cloudwatch_enterprise(account_id: &str, region: &str) {
    // Create custom CloudWatch dashboard for Amazon services
    let dashboard_config = r#"
{
  "widgets": [
    {
      "type": "metric",
      "properties": {
        "metrics": [
          ["AWS/EKS", "cluster_failed_request_count"],
          ["AWS/ECS", "CPUUtilization"],
          ["AWS/Lambda", "Duration"],
          ["AWS/Lambda", "Errors"]
        ],
        "period": 300,
        "stat": "Average",
        "region": "us-east-1",
        "title": "Amazon Core Services Health"
      }
    }
  ]
}
"#;
    
    write_file("/tmp/cloudwatch-dashboard.json", dashboard_config);
    exec(&format!("aws cloudwatch put-dashboard --dashboard-name Amazon-Enterprise-Dashboard --dashboard-body file:///tmp/cloudwatch-dashboard.json --region {}", region));
    
    // Create CloudWatch alarms
    setup_cloudwatch_alarms(region);
}

fn setup_cloudwatch_alarms(region: &str) {
    let alarms = vec![
        ("Amazon-High-CPU", "AWS/EKS", "CPUUtilization", "GreaterThanThreshold", "80"),
        ("Amazon-High-Memory", "AWS/ECS", "MemoryUtilization", "GreaterThanThreshold", "85"),
        ("Amazon-Lambda-Errors", "AWS/Lambda", "Errors", "GreaterThanThreshold", "10"),
        ("Amazon-ELB-Latency", "AWS/ApplicationELB", "TargetResponseTime", "GreaterThanThreshold", "2"),
    ];
    
    for (alarm_name, namespace, metric, comparison, threshold) in alarms {
        exec(&format!(r#"
aws cloudwatch put-metric-alarm \
    --alarm-name {} \
    --alarm-description "Amazon enterprise alert for {}" \
    --metric-name {} \
    --namespace {} \
    --statistic Average \
    --period 300 \
    --threshold {} \
    --comparison-operator {} \
    --evaluation-periods 2 \
    --alarm-actions arn:aws:sns:{region}:123456789012:amazon-alerts \
    --region {}
"#, alarm_name, metric, metric, namespace, threshold, comparison, region));
    }
}

fn setup_aws_security_hub(account_id: &str, regions: &[&str]) {
    // Enable Security Hub in all regions
    for region in regions {
        exec(&format!("aws securityhub enable-security-hub --region {}", region));
        
        // Enable foundational security standards
        exec(&format!(r#"
aws securityhub batch-enable-standards \
    --standards-subscription-requests StandardsArn=arn:aws:securityhub:::ruleset/finding-format/aws-foundational-security \
    --region {}
"#, region));
        
        // Enable AWS Config rules
        exec(&format!("aws configservice put-configuration-recorder --configuration-recorder name=amazon-config-recorder,roleARN=arn:aws:iam::{}:role/aws-config-role --region {}", account_id, region));
    }
    
    echo("🔒 AWS Security Hub enabled across all regions with enterprise compliance");
}

fn get_region_number(region: &str) -> &str {
    match region {
        "us-west-2" => "1",
        "eu-west-1" => "2", 
        "ap-southeast-1" => "3",
        _ => "4",
    }
}

// Additional Amazon-specific infrastructure functions
fn deploy_amazon_prime_infrastructure(regions: &[&str]) {
    echo("📦 Deploying Amazon Prime video and shipping infrastructure");
    
    for region in regions {
        // Deploy Prime Video streaming infrastructure
        deploy_prime_video_services(region);
        
        // Deploy Prime shipping logistics
        deploy_prime_logistics(region);
        
        // Deploy Prime membership services
        deploy_prime_membership_services(region);
    }
}

fn deploy_prime_video_services(region: &str) {
    let video_services = vec![
        "prime-video-streaming",
        "prime-video-transcoding",
        "prime-video-metadata",
        "prime-video-recommendations",
        "prime-video-drm",
    ];
    
    for service in video_services {
        deploy_ecs_service(region, service);
    }
}

fn deploy_prime_logistics(region: &str) {
    let logistics_services = vec![
        "prime-shipping-optimizer",
        "prime-delivery-tracking",
        "prime-warehouse-management",
        "prime-last-mile-delivery",
        "prime-drone-delivery",
    ];
    
    for service in logistics_services {
        deploy_ecs_service(region, service);
    }
}

fn deploy_amazon_retail_ecosystem(regions: &[&str]) {
    echo("🛒 Deploying Amazon retail ecosystem");
    
    for region in regions {
        // Deploy marketplace services
        deploy_marketplace_services(region);
        
        // Deploy fulfillment centers
        deploy_fulfillment_infrastructure(region);
        
        // Deploy advertising platform
        deploy_amazon_advertising_platform(region);
    }
}

fn deploy_marketplace_services(region: &str) {
    let marketplace_services = vec![
        "amazon-product-catalog",
        "amazon-seller-central",
        "amazon-reviews-system",
        "amazon-pricing-engine",
        "amazon-inventory-management",
        "amazon-fraud-detection",
    ];
    
    for service in marketplace_services {
        deploy_ecs_service(region, service);
    }
}

fn deploy_amazon_alexa_ecosystem(regions: &[&str]) {
    echo("🎤 Deploying Amazon Alexa voice ecosystem");
    
    for region in regions {
        // Deploy Alexa Voice Service
        deploy_alexa_voice_service(region);
        
        // Deploy Skills platform
        deploy_alexa_skills_platform(region);
        
        // Deploy Smart Home integration
        deploy_alexa_smart_home(region);
    }
}

fn deploy_alexa_voice_service(region: &str) {
    let alexa_config = format!(r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: amazon-alexa-avs-{}
spec:
  replicas: 10000
  selector:
    matchLabels:
      app: amazon-alexa-avs
      region: {}
  template:
    metadata:
      labels:
        app: amazon-alexa-avs
        region: {}
    spec:
      containers:
      - name: alexa-voice-service
        image: amazon/alexa-avs:latest
        ports:
        - containerPort: 8080
        - containerPort: 8443
        resources:
          requests:
            memory: "8Gi"
            cpu: "4000m"
          limits:
            memory: "16Gi"
            cpu: "8000m"
        env:
        - name: ALEXA_REGION
          value: "{}"
        - name: VOICE_PROCESSING_ENABLED
          value: "true"
        - name: NLU_MODEL_ENDPOINT
          value: "amazon-alexa-nlu-{}.default.svc.cluster.local:8080"
        - name: TTS_ENDPOINT
          value: "amazon-alexa-tts-{}.default.svc.cluster.local:8080"
"#, region, region, region, region, region, region);
    
    write_file(&format!("/tmp/alexa-avs-{}.yaml", region), &alexa_config);
    exec(&format!("kubectl apply -f /tmp/alexa-avs-{}.yaml", region));
}

fn setup_amazon_enterprise_monitoring(regions: &[&str]) {
    echo("📊 Setting up Amazon enterprise monitoring and observability");
    
    for region in regions {
        // Deploy X-Ray distributed tracing
        deploy_xray_tracing(region);
        
        // Deploy CloudWatch Insights
        deploy_cloudwatch_insights(region);
        
        // Deploy AWS Config for compliance
        deploy_aws_config_compliance(region);
    }
}

fn deploy_xray_tracing(region: &str) {
    let xray_config = format!(r#"
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: amazon-xray-daemon-{}
spec:
  selector:
    matchLabels:
      app: amazon-xray-daemon
      region: {}
  template:
    metadata:
      labels:
        app: amazon-xray-daemon
        region: {}
    spec:
      containers:
      - name: xray-daemon
        image: amazon/aws-xray-daemon:latest
        ports:
        - containerPort: 2000
          protocol: UDP
        resources:
          requests:
            memory: "512Mi"
            cpu: "100m"
          limits:
            memory: "1Gi"
            cpu: "200m"
        env:
        - name: AWS_REGION
          value: "{}"
        - name: _X_AMZN_TRACE_ID
          value: "Root=1-5e1b4151-5ac6c58f5a5b5c8f5a5b5c8f"
"#, region, region, region, region);
    
    write_file(&format!("/tmp/xray-daemon-{}.yaml", region), &xray_config);
    exec(&format!("kubectl apply -f /tmp/xray-daemon-{}.yaml", region));
}