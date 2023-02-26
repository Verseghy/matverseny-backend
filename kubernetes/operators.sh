curl -sL https://github.com/operator-framework/operator-lifecycle-manager/releases/download/v0.23.1/install.sh | bash -s v0.23.1
kubectl create -f https://operatorhub.io/install/strimzi-kafka-operator.yaml
kubectl create -f https://operatorhub.io/install/postgresql.yaml
# PLATFORM SPECIFIC
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.6.4/deploy/static/provider/do/deploy.yaml
# PLATFORM SPECIFIC END
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.11.0/cert-manager.yaml
sleep 10
kubectl annotate svc/ingress-nginx-controller -n ingress-nginx service.beta.kubernetes.io/do-loadbalancer-hostname=verseghy-gimnazium.net