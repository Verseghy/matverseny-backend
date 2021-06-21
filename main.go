package main

import (
	"context"
	"flag"
	"fmt"
	grpc_middleware "github.com/grpc-ecosystem/go-grpc-middleware"
	grpc_auth "github.com/grpc-ecosystem/go-grpc-middleware/auth"
	grpc_zap "github.com/grpc-ecosystem/go-grpc-middleware/logging/zap"
	"github.com/joho/godotenv"
	"github.com/mailgun/mailgun-go/v4"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/options"
	"go.uber.org/zap"
	"google.golang.org/grpc"
	"matverseny-backend/events"
	"matverseny-backend/handler"
	"matverseny-backend/jwt"
	"matverseny-backend/log"
	pb "matverseny-backend/proto"
	"net"
	"os"
	"time"
)

func envOrDefaultString(env, def string) string {
	if val, ok := os.LookupEnv(env); ok {
		return val
	}

	return def
}

func main() {
	flag.Parse()
	log.EnsureLogger()
	events.EnsureEvents()

	err := godotenv.Load()
	if err != nil {
		log.Logger.Fatal("Error loading .env file")
	}

	grpcListenAddr := envOrDefaultString("PORT", "6969")
	mongoAddr := envOrDefaultString("MONGO_URI", "mongodb://mongo1:27017,mongo2:27018,mongo3:27019/?replicaSet=rs0")
	mgDomain := envOrDefaultString("MAILGUN_DOMAIN", "sandboxd729d0f33df94a6999902985df8e0025.mailgun.org")
	mgKey := envOrDefaultString("MAILGUN_API_KEY", "set-it-in-env-file")

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	client, err := mongo.Connect(ctx, options.Client().ApplyURI(mongoAddr))
	if err != nil {
		log.Logger.Fatal("failed connecting to database", zap.Error(err))
	}

	mg := mailgun.NewMailgun(mgDomain, mgKey)

	authHandler := handler.NewAuthHandler(client, mg)
	competitionHandler := handler.NewCompetitionHandler(client)
	adminHandler := handler.NewAdminHandler(client)

	lis, err := net.Listen("tcp", fmt.Sprintf("0.0.0.0:%s", grpcListenAddr))
	if err != nil {
		log.Logger.Fatal("failed to listen", zap.Error(err))
	}
	log.Logger.Info(fmt.Sprintf("Listening on port: %s", grpcListenAddr))

	sopts := []grpc.ServerOption{
		grpc.StreamInterceptor(grpc_middleware.ChainStreamServer(
			grpc_zap.StreamServerInterceptor(log.Logger),
			grpc_auth.StreamServerInterceptor(jwt.ValidateAccessToken([]byte("test-key"))),
		)),
		grpc.UnaryInterceptor(grpc_middleware.ChainUnaryServer(
			grpc_zap.UnaryServerInterceptor(log.Logger),
			grpc_auth.UnaryServerInterceptor(jwt.ValidateAccessToken([]byte("test-key"))),
		)),
	}

	grpcServer := grpc.NewServer(sopts...)
	pb.RegisterAuthServer(grpcServer, authHandler)
	pb.RegisterCompetitionServer(grpcServer, competitionHandler)
	pb.RegisterAdminServer(grpcServer, adminHandler)

	// Run service
	if err := grpcServer.Serve(lis); err != nil {
		log.Logger.Fatal("couldn't serve grpcServer", zap.Error(err))
	}
}
