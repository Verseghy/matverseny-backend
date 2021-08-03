package int

import (
	"context"
	"github.com/dgrijalva/jwt-go"
	. "github.com/onsi/gomega"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/options"
	"google.golang.org/grpc/metadata"
	jwt2 "matverseny-backend/jwt"
	pb "matverseny-backend/proto"
	"time"
)

func cleanupMongo() {
	m, err := mongo.Connect(context.Background(), options.Client().ApplyURI("mongodb://mongo1:27017,mongo2:27018,mongo3:27019/?replicaSet=rs0"))
	Expect(err).To(BeNil())
	db := m.Database("comp")

	collections := []string{"auth", "solutions", "problems", "time", "teams"}
	for _, v := range collections {
		_, err := db.Collection(v).DeleteMany(context.Background(), bson.M{})
		Expect(err).To(BeNil())
	}
}

func registerUser(authClient pb.AuthClient) (string, string) {
	res, err := authClient.Register(context.Background(), &pb.RegisterRequest{
		Email:    "test@test.test",
		Password: "testtest",
		Name:     "test",
		School:   "test",
		Class:    0,
	})
	Expect(err).To(BeNil())

	Expect(res.RefreshToken).NotTo(BeNil())
	Expect(res.AccessToken).NotTo(BeNil())

	rt, err := jwt.ParseWithClaims(res.RefreshToken, &jwt2.RefreshClaims{}, func(token *jwt.Token) (interface{}, error) {
		return key, nil
	})
	Expect(err).To(BeNil())
	c, ok := rt.Claims.(*jwt2.RefreshClaims)
	Expect(ok).To(BeTrue())
	Expect(c.ExpiresAt).To(Satisfy(func(t int64) bool { return time.Now().Unix() < t }))
	Expect(c.UserID).NotTo(BeEmpty())
	Expect(c.IsAdmin).To(BeFalse())

	at, err := jwt.ParseWithClaims(res.AccessToken, &jwt2.AccessClaims{}, func(token *jwt.Token) (interface{}, error) {
		return key, nil
	})
	Expect(err).To(BeNil())
	c2, ok := at.Claims.(*jwt2.AccessClaims)
	Expect(ok).To(BeTrue())
	Expect(c2.ExpiresAt).To(Satisfy(func(t int64) bool { return time.Now().Unix() < t }))
	Expect(c2.UserID).NotTo(BeEmpty())
	Expect(c2.IsAdmin).To(BeFalse())
	Expect(c2.Team).To(BeEmpty())

	return res.RefreshToken, res.AccessToken
}

func refreshJWT(authClient pb.AuthClient, accesstoken string) (*jwt2.RefreshClaims, *jwt2.AccessClaims) {
	res, err := authClient.RefreshToken(context.Background(), &pb.RefreshTokenRequest{
		Token: accesstoken,
	})
	Expect(err).To(BeNil())

	Expect(res.Token).NotTo(BeNil())

	rt, err := jwt.ParseWithClaims(accesstoken, &jwt2.RefreshClaims{}, func(token *jwt.Token) (interface{}, error) {
		return key, nil
	})
	Expect(err).To(BeNil())
	c, ok := rt.Claims.(*jwt2.RefreshClaims)
	Expect(ok).To(BeTrue())
	Expect(c.ExpiresAt).To(Satisfy(func(t int64) bool { return time.Now().Unix() < t }))
	Expect(c.UserID).NotTo(BeEmpty())

	at, err := jwt.ParseWithClaims(res.Token, &jwt2.AccessClaims{}, func(token *jwt.Token) (interface{}, error) {
		return key, nil
	})
	Expect(err).To(BeNil())
	c2, ok := at.Claims.(*jwt2.AccessClaims)
	Expect(ok).To(BeTrue())
	Expect(c2.ExpiresAt).To(Satisfy(func(t int64) bool { return time.Now().Unix() < t }))
	Expect(c2.UserID).NotTo(BeEmpty())

	return c, c2
}

func authenticatedContext(at string) context.Context {
	return metadata.AppendToOutgoingContext(context.Background(), "authorization", at)
}
