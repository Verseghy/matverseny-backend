#!/usr/bin/env bash
set -e

./operators.sh
sleep 30
kubectl apply -f cluster-issuer.yaml
kubectl apply -f configmap-ingress-nginx.yaml

kubectl apply -f namespace-matverseny.yaml
kubectl apply -f ./matverseny-backend/ingress.yaml
kubectl apply -f ./matverseny-backend/kafka-1.yaml
kubectl apply -f ./matverseny-backend/postgres-1.yaml
kubectl apply -f ./matverseny-backend/secret.yaml

kubectl apply -f namespace-iam.yaml
envsubst < ./iam/secret.yaml | kubectl apply -f -
kubectl apply -f ./iam/mysql.yaml
sleep 10
kubectl apply -f ./iam/migration.yaml
kubectl apply -f ./iam/iam.yaml
kubectl apply -f ./iam/ingress.yaml

directory=$(mktemp -d)
pushd "$directory"
openssl genrsa -out keypair.pem 4096
openssl rsa -in keypair.pem -pubout -out publickey.crt
export JWT_PRIVATE=$(cat keypair.pem)
export JWT_PUBLIC=$(cat publickey.crt)
shred -uz keypair.pem
popd
echo $JWT_PRIVATE
echo $JWT_PUBLIC
# TODO: fix this
# envsubst < ./iam/jwt.yaml | kubectl apply -f -
unset JWT_PRIVATE

echo please fill in the secret for IAM_APP_SECRET
sleep 30
kubectl apply -f ./matverseny-backend/matverseny-backend.yaml
kubectl apply -f ./matverseny-backend/matverseny-backend-migration.yaml
