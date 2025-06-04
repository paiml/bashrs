// Amazon EC2 Auto Scaling at enterprise scale with Rash
// Demonstrates massive EC2 fleet management and auto-scaling

#[rash::main]
fn amazon_ec2_enterprise_autoscaling() {
    let regions = vec!["us-east-1", "us-west-2", "eu-west-1", "ap-southeast-1"];
    let peak_capacity = "1000000"; // 1 million EC2 instances at peak
    let instance_types = vec!["m5.24xlarge", "c5.24xlarge", "r5.24xlarge", "i3.24xlarge"];
    
    echo("🚀 Amazon EC2 Enterprise Auto Scaling deployment");
    
    // Deploy massive auto scaling groups
    deploy_enterprise_autoscaling_groups(&regions, &instance_types, &peak_capacity);
    
    // Setup predictive scaling with machine learning
    deploy_predictive_scaling_system(&regions);
    
    // Configure spot fleet management
    deploy_spot_fleet_management(&regions);
    
    // Setup cross-zone load balancing
    deploy_cross_zone_load_balancing(&regions);
    
    // Configure capacity reservations
    deploy_capacity_reservations(&regions);
    
    // Setup instance lifecycle management
    deploy_instance_lifecycle_management(&regions);
    
    echo("✅ Amazon EC2 enterprise auto scaling deployment completed");
}

fn deploy_enterprise_autoscaling_groups(regions: &[&str], instance_types: &[&str], capacity: &str) {
    echo(&format!("📈 Deploying auto scaling groups for {} instances", capacity));
    
    for region in regions {
        for instance_type in instance_types {
            create_autoscaling_group(region, instance_type, capacity);
        }
        
        // Create launch templates for each workload type
        create_workload_launch_templates(region);
        
        // Setup scaling policies
        configure_scaling_policies(region);
    }
}

fn create_autoscaling_group(region: &str, instance_type: &str, max_capacity: &str) {
    let asg_name = format!("amazon-{}-asg-{}", region, instance_type.replace(".", "-"));
    let min_size = "10000"; // Minimum 10k instances per ASG
    let desired_capacity = "50000"; // 50k instances normally
    
    // Create launch template
    let launch_template_config = format!(r#"
aws ec2 create-launch-template \
    --launch-template-name {}-template \
    --launch-template-data '{{
        "ImageId": "ami-0abcdef1234567890",
        "InstanceType": "{}",
        "KeyName": "amazon-production-key",
        "SecurityGroupIds": ["sg-amazon-web", "sg-amazon-app"],
        "UserData": "{}",
        "IamInstanceProfile": {{
            "Name": "AmazonEC2ServiceRole"
        }},
        "BlockDeviceMappings": [{{
            "DeviceName": "/dev/xvda",
            "Ebs": {{
                "VolumeSize": 500,
                "VolumeType": "gp3",
                "Iops": 16000,
                "Throughput": 1000,
                "DeleteOnTermination": true,
                "Encrypted": true
            }}
        }}],
        "Monitoring": {{
            "Enabled": true
        }},
        "MetadataOptions": {{
            "HttpTokens": "required",
            "HttpPutResponseHopLimit": 2,
            "HttpEndpoint": "enabled"
        }}
    }}' \
    --region {}
"#, asg_name, instance_type, get_userdata_script(), region);
    
    exec(&launch_template_config);
    
    // Create auto scaling group
    let asg_config = format!(r#"
aws autoscaling create-auto-scaling-group \
    --auto-scaling-group-name {} \
    --launch-template 'LaunchTemplateName={}-template,Version=$Latest' \
    --min-size {} \
    --max-size {} \
    --desired-capacity {} \
    --vpc-zone-identifier "subnet-1a,subnet-1b,subnet-1c" \
    --health-check-type ELB \
    --health-check-grace-period 300 \
    --default-cooldown 300 \
    --termination-policies "OldestInstance,ClosestToNextInstanceHour" \
    --capacity-rebalance \
    --tags 'Key=Name,Value=Amazon-Production-{},PropagateAtLaunch=true' 'Key=Environment,Value=Production,PropagateAtLaunch=true' 'Key=Team,Value=Amazon-Infrastructure,PropagateAtLaunch=true' \
    --region {}
"#, asg_name, asg_name, min_size, max_capacity, desired_capacity, instance_type, region);
    
    exec(&asg_config);
}

fn get_userdata_script() -> &'static str {
    "IyEvYmluL2Jhc2gKCiMgQW1hem9uIEVDMiBVc2VyIERhdGEgU2NyaXB0CmVjaG8gIlN0YXJ0aW5nIEFtYXpvbiBFQzIgaW5zdGFuY2UgaW5pdGlhbGl6YXRpb24uLi4iCgojIEluc3RhbGwgQ2xvdWRXYXRjaCBhZ2VudAp3Z2V0IC1PIC90bXAvYW1hem9uLWNsb3Vkd2F0Y2gtYWdlbnQucnBtIGh0dHBzOi8vczMuYW1hem9uYXdzLmNvbS9hbWF6b25jbG91ZHdhdGNoLWFnZW50L2FtYXpvbl9saW51eC9hbWQ2NC9sYXRlc3QvYW1hem9uLWNsb3Vkd2F0Y2gtYWdlbnQucnBtCnl1bSBpbnN0YWxsIC15IC90bXAvYW1hem9uLWNsb3Vkd2F0Y2gtYWdlbnQucnBtCgojIENvbmZpZ3VyZSBDbG91ZFdhdGNoIGFnZW50CmNhdCA+IC9vcHQvYXdzL2FtYXpvbi1jbG91ZHdhdGNoLWFnZW50L2V0Yy9hbWF6b24tY2xvdWR3YXRjaC1hZ2VudC5qc29uIDw8RU9GCnsKICAgICJhZ2VudCI6IHsKICAgICAgICAibWV0cmljc19jb2xsZWN0aW9uX2ludGVydmFsIjogNjAsCiAgICAgICAgInJ1bl9hc191c2VyIjogImN3YWdlbnQiCiAgICB9LAogICAgIm1ldHJpY3MiOiB7CiAgICAgICAgIm5hbWVzcGFjZSI6ICJBbWF6b24vRUMyIiwKICAgICAgICAibWV0cmljc19jb2xsZWN0ZWQiOiB7CiAgICAgICAgICAgICJjcHUiOiB7CiAgICAgICAgICAgICAgICAibWVhc3VyZW1lbnQiOiBbCiAgICAgICAgICAgICAgICAgICAgImNwdV91c2FnZV9pZGxlIiwKICAgICAgICAgICAgICAgICAgICAiY3B1X3VzYWdlX2lvd2FpdCIsCiAgICAgICAgICAgICAgICAgICAgImNwdV91c2FnZV91c2VyIiwKICAgICAgICAgICAgICAgICAgICAiY3B1X3VzYWdlX3N5c3RlbSIKICAgICAgICAgICAgICAgIF0sCiAgICAgICAgICAgICAgICAibWV0cmljc19jb2xsZWN0aW9uX2ludGVydmFsIjogNjAKICAgICAgICAgICAgfSwKICAgICAgICAgICAgImRpc2siOiB7CiAgICAgICAgICAgICAgICAibWVhc3VyZW1lbnQiOiBbCiAgICAgICAgICAgICAgICAgICAgInVzZWRfcGVyY2VudCIKICAgICAgICAgICAgICAgIF0sCiAgICAgICAgICAgICAgICAibWV0cmljc19jb2xsZWN0aW9uX2ludGVydmFsIjogNjAsCiAgICAgICAgICAgICAgICAicmVzb3VyY2VzIjogWwogICAgICAgICAgICAgICAgICAgICIqIgogICAgICAgICAgICAgICAgXQogICAgICAgICAgICB9LAogICAgICAgICAgICAibWVtIjogewogICAgICAgICAgICAgICAgIm1lYXN1cmVtZW50IjogWwogICAgICAgICAgICAgICAgICAgICJtZW1fdXNlZF9wZXJjZW50IgogICAgICAgICAgICAgICAgXSwKICAgICAgICAgICAgICAgICJtZXRyaWNzX2NvbGxlY3Rpb25faW50ZXJ2YWwiOiA2MAogICAgICAgICAgICB9CiAgICAgICAgfQogICAgfQp9CkVPRgoKIyBTdGFydCBDbG91ZFdhdGNoIGFnZW50CnN5c3RlbWN0bCBlbmFibGUgYW1hem9uLWNsb3Vkd2F0Y2gtYWdlbnQKc3lzdGVtY3RsIHN0YXJ0IGFtYXpvbi1jbG91ZHdhdGNoLWFnZW50CgojIEluc3RhbGwgYW5kIGNvbmZpZ3VyZSBEb2NrZXIKeXVtIGluc3RhbGwgLXkgZG9ja2VyCnN5c3RlbWN0bCBlbmFibGUgZG9ja2VyCnN5c3RlbWN0bCBzdGFydCBkb2NrZXIKdXNlcm1vZCAtYSAtRyBkb2NrZXIgZWMyLXVzZXIKCiMgSW5zdGFsbCBhbmQgY29uZmlndXJlIEt1YmVybmV0ZXMgKEVLUykKY3VybCAtbyAvaWF3cy1pYW0tYXV0aGVudGljYXRvciBodHRwczovL2FtYXpvbi1la3MuczMudXMtd2VzdC0yLmFtYXpvbmF3cy5jb20vMS4yMS41LzIwMjMtMDEtMzAvYmluL2xpbnV4L2FtZDY0L2F3cy1pYW0tYXV0aGVudGljYXRvcgpjaG1vZCAreCAvYXdzLWlhbS1hdXRoZW50aWNhdG9yCm12IC9hd3MtaWFtLWF1dGhlbnRpY2F0b3IgL3Vzci9sb2NhbC9iaW4vCgojIEluaXRpYWxpemUgRUtTIG5vZGUKL2V0Yy9la3MvYm9vdHN0cmFwLnNoIGFtYXpvbi1lbnRlcnByaXNlLWVrcwoKZWNobyAiQW1hem9uIEVDMiBpbnN0YW5jZSBpbml0aWFsaXphdGlvbiBjb21wbGV0ZWQiCg=="
}

fn create_workload_launch_templates(region: &str) {
    let workload_types = vec![
        ("web-tier", "m5.2xlarge", "Amazon web server tier"),
        ("app-tier", "c5.4xlarge", "Amazon application tier"),
        ("cache-tier", "r5.2xlarge", "Amazon cache tier"),
        ("db-tier", "r5.8xlarge", "Amazon database tier"),
        ("ml-tier", "p3.8xlarge", "Amazon ML inference tier"),
    ];
    
    for (workload, instance_type, description) in workload_types {
        let template_config = format!(r#"
aws ec2 create-launch-template \
    --launch-template-name amazon-{}-template-{} \
    --launch-template-data '{{
        "ImageId": "ami-0abcdef1234567890",
        "InstanceType": "{}",
        "KeyName": "amazon-production-key",
        "SecurityGroupIds": ["sg-amazon-{}", "sg-amazon-common"],
        "UserData": "{}",
        "IamInstanceProfile": {{
            "Name": "AmazonEC2{}Role"
        }},
        "TagSpecifications": [{{
            "ResourceType": "instance",
            "Tags": [
                {{"Key": "Name", "Value": "Amazon-{}-{}"}},
                {{"Key": "Workload", "Value": "{}"}},
                {{"Key": "Environment", "Value": "Production"}},
                {{"Key": "Team", "Value": "Amazon-Infrastructure"}}
            ]
        }}]
    }}' \
    --region {}
"#, workload, region, instance_type, workload, get_userdata_script(), workload, workload, region, workload, region);
        
        exec(&template_config);
    }
}

fn configure_scaling_policies(region: &str) {
    let scaling_policies = vec![
        ("scale-up-cpu", "ChangeInCapacity", "5000", "GreaterThanThreshold", "70"),
        ("scale-down-cpu", "ChangeInCapacity", "-2000", "LessThanThreshold", "30"),
        ("scale-up-memory", "ChangeInCapacity", "3000", "GreaterThanThreshold", "80"),
        ("scale-down-memory", "ChangeInCapacity", "-1000", "LessThanThreshold", "40"),
        ("scale-up-network", "ChangeInCapacity", "2000", "GreaterThanThreshold", "500000000"), // 500 MB/s
    ];
    
    for (policy_name, adjustment_type, scaling_adjustment, comparison, threshold) in scaling_policies {
        let policy_config = format!(r#"
aws autoscaling put-scaling-policy \
    --auto-scaling-group-name amazon-{}-asg-primary \
    --policy-name amazon-{}-{} \
    --policy-type StepScaling \
    --adjustment-type {} \
    --step-adjustments 'MetricIntervalLowerBound=0,ScalingAdjustment={}' \
    --min-adjustment-magnitude 100 \
    --cooldown 300 \
    --region {}
"#, region, region, policy_name, adjustment_type, scaling_adjustment, region);
        
        exec(&policy_config);
        
        // Create CloudWatch alarm for the scaling policy
        let alarm_config = format!(r#"
aws cloudwatch put-metric-alarm \
    --alarm-name amazon-{}-{}-alarm \
    --alarm-description "Amazon {} scaling alarm" \
    --metric-name CPUUtilization \
    --namespace AWS/EC2 \
    --statistic Average \
    --period 300 \
    --threshold {} \
    --comparison-operator {} \
    --evaluation-periods 2 \
    --alarm-actions $(aws autoscaling describe-policies --auto-scaling-group-name amazon-{}-asg-primary --policy-names amazon-{}-{} --query 'ScalingPolicies[0].PolicyARN' --output text) \
    --dimensions Name=AutoScalingGroupName,Value=amazon-{}-asg-primary \
    --region {}
"#, region, policy_name, policy_name, threshold, comparison, region, region, policy_name, region, region);
        
        exec(&alarm_config);
    }
}

fn deploy_predictive_scaling_system(regions: &[&str]) {
    echo("🔮 Deploying predictive scaling with machine learning");
    
    for region in regions {
        // Enable predictive scaling for auto scaling groups
        enable_predictive_scaling(region);
        
        // Deploy custom ML models for capacity prediction
        deploy_capacity_prediction_models(region);
        
        // Setup forecast data ingestion
        setup_forecast_data_pipeline(region);
    }
}

fn enable_predictive_scaling(region: &str) {
    let predictive_config = format!(r#"
aws autoscaling put-scaling-policy \
    --auto-scaling-group-name amazon-{}-asg-primary \
    --policy-name amazon-{}-predictive-scaling \
    --policy-type PredictiveScaling \
    --predictive-scaling-configuration '{{
        "MetricSpecifications": [{{
            "TargetValue": 70,
            "PredefinedMetricSpecification": {{
                "PredefinedMetricType": "ASGAverageCPUUtilization"
            }}
        }}],
        "Mode": "ForecastAndScale",
        "SchedulingBufferTime": 300,
        "MaxCapacityBreachBehavior": "IncreaseMaxCapacity",
        "MaxCapacityBuffer": 10
    }}' \
    --region {}
"#, region, region, region);
    
    exec(&predictive_config);
}

fn deploy_capacity_prediction_models(region: &str) {
    let ml_config = format!(r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: amazon-capacity-predictor-{}
spec:
  replicas: 10
  selector:
    matchLabels:
      app: amazon-capacity-predictor
      region: {}
  template:
    metadata:
      labels:
        app: amazon-capacity-predictor
        region: {}
    spec:
      containers:
      - name: capacity-predictor
        image: amazon/capacity-predictor:latest
        ports:
        - containerPort: 8080
        resources:
          requests:
            memory: "8Gi"
            cpu: "4000m"
            nvidia.com/gpu: 1
          limits:
            memory: "16Gi"
            cpu: "8000m"
            nvidia.com/gpu: 2
        env:
        - name: PREDICTION_REGION
          value: "{}"
        - name: MODEL_TYPE
          value: "LSTM_TIME_SERIES"
        - name: FORECAST_HORIZON
          value: "168" # 7 days in hours
        - name: TRAINING_WINDOW
          value: "8760" # 1 year in hours
        - name: METRICS_SOURCES
          value: "cloudwatch,prometheus,custom"
        - name: S3_MODEL_BUCKET
          value: "amazon-ml-models-{}"
        - name: SAGEMAKER_ENDPOINT
          value: "amazon-capacity-prediction-{}"
"#, region, region, region, region, region, region);
    
    write_file(&format!("/tmp/capacity-predictor-{}.yaml", region), &ml_config);
    exec(&format!("kubectl apply -f /tmp/capacity-predictor-{}.yaml", region));
}

fn deploy_spot_fleet_management(regions: &[&str]) {
    echo("💰 Deploying Spot Fleet management for cost optimization");
    
    for region in regions {
        // Create spot fleet requests
        create_spot_fleet_requests(region);
        
        // Setup spot instance lifecycle management
        setup_spot_lifecycle_management(region);
        
        // Configure mixed instance types
        configure_mixed_instance_policies(region);
    }
}

fn create_spot_fleet_requests(region: &str) {
    let spot_fleet_config = format!(r#"
aws ec2 request-spot-fleet \
    --spot-fleet-request-config '{{
        "IamFleetRole": "arn:aws:iam::123456789012:role/aws-ec2-spot-fleet-tagging-role",
        "AllocationStrategy": "diversified",
        "TargetCapacity": 100000,
        "SpotPrice": "0.50",
        "LaunchSpecifications": [
            {{
                "ImageId": "ami-0abcdef1234567890",
                "InstanceType": "m5.large",
                "KeyName": "amazon-production-key",
                "SecurityGroups": [{{
                    "GroupId": "sg-amazon-spot"
                }}],
                "SubnetId": "subnet-12345",
                "UserData": "{}",
                "WeightedCapacity": 1.0
            }},
            {{
                "ImageId": "ami-0abcdef1234567890",
                "InstanceType": "m5.xlarge",
                "KeyName": "amazon-production-key",
                "SecurityGroups": [{{
                    "GroupId": "sg-amazon-spot"
                }}],
                "SubnetId": "subnet-67890",
                "UserData": "{}",
                "WeightedCapacity": 2.0
            }},
            {{
                "ImageId": "ami-0abcdef1234567890",
                "InstanceType": "c5.large",
                "KeyName": "amazon-production-key",
                "SecurityGroups": [{{
                    "GroupId": "sg-amazon-spot"
                }}],
                "SubnetId": "subnet-abcde",
                "UserData": "{}",
                "WeightedCapacity": 1.0
            }}
        ],
        "TerminateInstancesWithExpiration": true,
        "Type": "maintain",
        "ReplaceUnhealthyInstances": true,
        "InstanceInterruptionBehavior": "terminate",
        "TagSpecifications": [{{
            "ResourceType": "spot-fleet-request",
            "Tags": [
                {{"Key": "Name", "Value": "Amazon-Spot-Fleet-{}"}},
                {{"Key": "Environment", "Value": "Production"}},
                {{"Key": "Team", "Value": "Amazon-Infrastructure"}}
            ]
        }}]
    }}' \
    --region {}
"#, get_userdata_script(), get_userdata_script(), get_userdata_script(), region, region);
    
    exec(&spot_fleet_config);
}

fn configure_mixed_instance_policies(region: &str) {
    let mixed_instance_config = format!(r#"
aws autoscaling create-auto-scaling-group \
    --auto-scaling-group-name amazon-{}-mixed-instances \
    --min-size 5000 \
    --max-size 200000 \
    --desired-capacity 25000 \
    --vpc-zone-identifier "subnet-1a,subnet-1b,subnet-1c" \
    --mixed-instances-policy '{{
        "LaunchTemplate": {{
            "LaunchTemplateSpecification": {{
                "LaunchTemplateName": "amazon-mixed-template-{}",
                "Version": "$Latest"
            }},
            "Overrides": [
                {{"InstanceType": "m5.large", "WeightedCapacity": "1"}},
                {{"InstanceType": "m5.xlarge", "WeightedCapacity": "2"}},
                {{"InstanceType": "m5.2xlarge", "WeightedCapacity": "4"}},
                {{"InstanceType": "c5.large", "WeightedCapacity": "1"}},
                {{"InstanceType": "c5.xlarge", "WeightedCapacity": "2"}},
                {{"InstanceType": "r5.large", "WeightedCapacity": "1"}},
                {{"InstanceType": "r5.xlarge", "WeightedCapacity": "2"}}
            ]
        }},
        "InstancesDistribution": {{
            "OnDemandAllocationStrategy": "prioritized",
            "OnDemandBaseCapacity": 5000,
            "OnDemandPercentageAboveBaseCapacity": 20,
            "SpotAllocationStrategy": "capacity-optimized",
            "SpotInstancePools": 10,
            "SpotMaxPrice": "0.50"
        }}
    }}' \
    --tags 'Key=Name,Value=Amazon-Mixed-Instances-{},PropagateAtLaunch=true' 'Key=Environment,Value=Production,PropagateAtLaunch=true' \
    --region {}
"#, region, region, region, region);
    
    exec(&mixed_instance_config);
}

fn deploy_cross_zone_load_balancing(regions: &[&str]) {
    echo("⚖️ Deploying cross-zone load balancing");
    
    for region in regions {
        // Create Application Load Balancers
        create_application_load_balancers(region);
        
        // Create Network Load Balancers
        create_network_load_balancers(region);
        
        // Configure Global Load Balancer
        configure_global_load_balancer(region);
    }
}

fn create_application_load_balancers(region: &str) {
    let alb_config = format!(r#"
aws elbv2 create-load-balancer \
    --name amazon-enterprise-alb-{} \
    --subnets subnet-1a subnet-1b subnet-1c \
    --security-groups sg-amazon-alb \
    --scheme internet-facing \
    --type application \
    --ip-address-type ipv4 \
    --tags 'Key=Name,Value=Amazon-Enterprise-ALB-{}' 'Key=Environment,Value=Production' 'Key=Team,Value=Amazon-Infrastructure' \
    --region {}
"#, region, region, region);
    
    exec(&alb_config);
    
    // Create target groups for different services
    let services = vec!["web", "api", "mobile", "admin"];
    for service in services {
        let target_group_config = format!(r#"
aws elbv2 create-target-group \
    --name amazon-{}-{}-tg \
    --protocol HTTP \
    --port 80 \
    --vpc-id vpc-amazon-{} \
    --health-check-enabled \
    --health-check-interval-seconds 30 \
    --health-check-path /{}/health \
    --health-check-protocol HTTP \
    --health-check-timeout-seconds 10 \
    --healthy-threshold-count 3 \
    --unhealthy-threshold-count 3 \
    --target-type instance \
    --tags 'Key=Name,Value=Amazon-{}-{}-TargetGroup' 'Key=Service,Value={}' \
    --region {}
"#, service, region, region, service, service, region, service, region);
        
        exec(&target_group_config);
    }
}

fn deploy_capacity_reservations(regions: &[&str]) {
    echo("🔒 Deploying EC2 Capacity Reservations");
    
    for region in regions {
        // Create capacity reservations for critical workloads
        create_capacity_reservations(region);
        
        // Setup reservation monitoring
        setup_reservation_monitoring(region);
    }
}

fn create_capacity_reservations(region: &str) {
    let critical_workloads = vec![
        ("web-tier", "m5.2xlarge", "10000"),
        ("api-tier", "c5.4xlarge", "5000"),
        ("database-tier", "r5.8xlarge", "2000"),
        ("cache-tier", "r5.2xlarge", "3000"),
    ];
    
    for (workload, instance_type, capacity) in critical_workloads {
        let reservation_config = format!(r#"
aws ec2 create-capacity-reservation \
    --instance-type {} \
    --instance-platform Linux/UNIX \
    --availability-zone {}-1a \
    --instance-count {} \
    --end-date-type unlimited \
    --instance-match-criteria targeted \
    --tag-specifications 'ResourceType=capacity-reservation,Tags=[{{Key=Name,Value=Amazon-{}-{}-Reservation}},{{Key=Workload,Value={}}},{{Key=Environment,Value=Production}}]' \
    --region {}
"#, instance_type, region, capacity, workload, region, workload, region);
        
        exec(&reservation_config);
    }
}

fn deploy_instance_lifecycle_management(regions: &[&str]) {
    echo("🔄 Deploying instance lifecycle management");
    
    for region in regions {
        // Setup instance refresh policies
        configure_instance_refresh(region);
        
        // Deploy warm pools
        configure_warm_pools(region);
        
        // Setup automated patching
        setup_automated_patching(region);
    }
}

fn configure_instance_refresh(region: &str) {
    let refresh_config = format!(r#"
aws autoscaling start-instance-refresh \
    --auto-scaling-group-name amazon-{}-asg-primary \
    --preferences '{{
        "InstanceWarmup": 300,
        "MinHealthyPercentage": 90,
        "CheckpointPercentages": [20, 50, 100],
        "CheckpointDelay": 3600,
        "SkipMatching": false
    }}' \
    --desired-configuration '{{
        "LaunchTemplate": {{
            "LaunchTemplateName": "amazon-latest-template-{}",
            "Version": "$Latest"
        }},
        "MixedInstancesPolicy": {{
            "LaunchTemplate": {{
                "LaunchTemplateSpecification": {{
                    "LaunchTemplateName": "amazon-latest-template-{}",
                    "Version": "$Latest"
                }}
            }}
        }}
    }}' \
    --region {}
"#, region, region, region, region);
    
    exec(&refresh_config);
}

fn configure_warm_pools(region: &str) {
    let warm_pool_config = format!(r#"
aws autoscaling put-warm-pool \
    --auto-scaling-group-name amazon-{}-asg-primary \
    --max-group-prepared-capacity 50000 \
    --min-size 10000 \
    --pool-state Stopped \
    --instance-reuse-policy ReuseOnScaleIn=true \
    --region {}
"#, region, region);
    
    exec(&warm_pool_config);
}