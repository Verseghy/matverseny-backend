./operators.sh
sleep 10
kubectl apply -f cluster-issuer.yaml
kubectl apply -f configmap-ingress-nginx.yaml
kubectl apply -f namespace-matverseny-backend.yaml
kubectl apply -f ./matverseny-backend
