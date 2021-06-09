package int_test

import (
	"context"
	"fmt"
	. "github.com/onsi/ginkgo"
	. "github.com/onsi/gomega"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/options"
	"google.golang.org/grpc"
	"google.golang.org/grpc/metadata"
	pb "matverseny-backend/proto"
)

var _ = Describe("Competition", func() {
	m, err := mongo.Connect(context.Background(), options.Client().ApplyURI("mongodb://mongo1:27017,mongo2:27018,mongo3:27019/?replicaSet=rs0"))
	Expect(err).To(BeNil())
	db := m.Database("comp")

	collections := []string{"auth", "solutions", "problems", "time"}
	for _, v := range collections {
		_, err := db.Collection(v).DeleteMany(context.Background(), bson.M{})
		Expect(err).To(BeNil())
	}

	var opts []grpc.DialOption
	opts = append(opts, grpc.WithInsecure())

	conn, err := grpc.Dial("localhost:6969", opts...)
	Expect(err).To(BeNil())

	authClient := pb.NewAuthClient(conn)
	r, err := authClient.Register(context.Background(), &pb.RegisterRequest{
		Email:    "test@test.test",
		Password: "password",
		Name:     "name",
		School:   "school",
		Class:    0,
	})
	Expect(err).To(BeNil())

	client := pb.NewCompetitionClient(conn)

	It("test", func() {
		ctx := context.Background()
		ctx = metadata.AppendToOutgoingContext(ctx, "authorization", "Bearer "+r.AccessToken)
		cl, err := client.GetProblems(ctx, &pb.GetProblemsRequest{})
		Expect(err).To(BeNil())

		for {
			data, err := cl.Recv()
			if err != nil {
				fmt.Println(err)
				break
			}
			fmt.Println(data)
		}

	})
})
