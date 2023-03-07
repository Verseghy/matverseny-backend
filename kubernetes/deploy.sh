#!/usr/bin/env bash

./operators.sh
sleep 10
kubectl apply -f cluster-issuer.yaml
kubectl apply -f configmap-ingress-nginx.yaml
kubectl apply -f namespace-matverseny-backend.yaml
kubectl apply -f ./matverseny-backend/ingress.yaml
kubectl apply -f ./matverseny-backend/kafka-1.yaml
kubectl apply -f ./matverseny-backend/postgres-1.yaml
kubectl apply -f ./matverseny-backend/matverseny-backend.yaml
kubectl apply -f ./matverseny-backend/matverseny-backend-migration.yaml
