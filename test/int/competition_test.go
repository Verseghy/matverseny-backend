package int_test

import (
	"context"
	"fmt"
	. "github.com/onsi/ginkgo"
	. "github.com/onsi/gomega"
	"google.golang.org/grpc"
	"google.golang.org/grpc/metadata"
	"math/rand"
	pb "matverseny-backend/proto"
	"sync"
	"time"
)

var _ = Describe("Competition", func() {
	/*m, err := mongo.Connect(context.Background(), options.Client().ApplyURI("mongodb://mongo1:27017,mongo2:27018,mongo3:27019/?replicaSet=rs0"))
	Expect(err).To(BeNil())
	db := m.Database("comp")

	collections := []string{"auth", "solutions", "problems", "time"}
	for _, v := range collections {
		_, err := db.Collection(v).DeleteMany(context.Background(), bson.M{})
		Expect(err).To(BeNil())
	}*/

	var opts []grpc.DialOption
	opts = append(opts, grpc.WithInsecure())

	conn, err := grpc.Dial("localhost:6969", opts...)
	Expect(err).To(BeNil())

	authClient := pb.NewAuthClient(conn)

	emails := []string{"test@test.test", "test@test.test2", "test@test.test3", "test@test.test4", "test@test.test5", "test@test.test6"}
	r := []*pb.LoginResponse{}

	for _, v := range emails {
		re, err := authClient.Login(context.Background(), &pb.LoginRequest{
			Email:    v,
			Password: "testtest",
		})
		Expect(err).To(BeNil())
		r = append(r, re)
	}

	Expect(err).To(BeNil())

	getCtx := func(w uint32) context.Context {
		ctx := context.Background()
		ctx = metadata.AppendToOutgoingContext(ctx, "Authorization", r[w].AccessToken)
		return ctx
	}

	client := pb.NewCompetitionClient(conn)

	getProblems := func(w uint32) []*pb.Problem {
		cl, err := client.GetProblems(getCtx(w), &pb.ProblemStreamRequest{})
		Expect(err).To(BeNil())

		p := make([]*pb.Problem, 0, 193)

		for i := 0; i < 193; i++ {
			res, err := cl.Recv()
			Expect(err).To(BeNil())

			p = append(p, res.Initial.Problem)
		}

		return p
	}

	getSolutionStream := func(w uint32) <-chan *pb.GetSolutionsResponse {
		cl, err := client.GetSolutions(getCtx(w), &pb.GetSolutionsRequest{})
		Expect(err).To(BeNil())

		ch := make(chan *pb.GetSolutionsResponse)
		go func() {
			defer GinkgoRecover()

			for {
				res, err := cl.Recv()
				Expect(err).To(BeNil())

				ch <- res
			}
		}()

		return ch
	}

	setSolution := func(problemID string, value int64, w uint32) {
		_, err := client.SetSolutions(getCtx(w), &pb.SetSolutionsRequest{
			Id:     problemID,
			Value:  value,
			Delete: false,
		})
		Expect(err).To(BeNil())
	}

	It("test", func() {
		rand.Seed(time.Now().UnixNano())
		p := getProblems(0)

		buff := make(chan struct{}, 20)

		var wg sync.WaitGroup
		var wg2 sync.WaitGroup
		num := 500
		num2 := 193
		wg.Add(num)
		wg2.Add(num)
		for i := 0; i < num; i++ {
			buff <- struct{}{}
			go func() {
				defer wg.Done()
				defer func() { <-buff }()

				ch := getSolutionStream(uint32(i % len(emails)))
				go func() {
					defer GinkgoRecover()

					for j := 0; j < num2; j++ {
						v := <-ch
						_, err := fmt.Fprintf(GinkgoWriter, `Worker [%d]: Got solution for ID "%s" %d`, i, v.Id, v.Value)
						Expect(err).To(BeNil())
					}
					wg2.Done()
				}()

				for j := 0; j < num2; j++ {
					f := rand.Intn(192)
					d := rand.Intn(10000)
					setSolution(p[f].Id, int64(d), uint32(i%len(emails)))
					fmt.Fprintf(GinkgoWriter, `Worker [%d]: Set solution for ID "%s" %d`, i, p[f].Id, d)
				}
			}()
		}
		fmt.Fprintf(GinkgoWriter, `Waiting on wg1`)
		wg.Wait()
		fmt.Fprintf(GinkgoWriter, `Waiting on wg2`)
		wg2.Wait()
	})
})
