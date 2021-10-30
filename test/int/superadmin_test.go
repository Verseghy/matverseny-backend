package int

import (
	"context"
	"google.golang.org/grpc/metadata"
	"matverseny-backend/internal/superadmin"
	pb "matverseny-backend/proto"
	"time"

	. "github.com/onsi/ginkgo"
	. "github.com/onsi/gomega"
	"google.golang.org/grpc"
)

var _ = Describe("Superadmin", func() {
	var opts []grpc.DialOption
	opts = append(opts, grpc.WithInsecure())

	conn, err := grpc.Dial("localhost:6969", opts...)
	Expect(err).To(BeNil())

	superadminClient := pb.NewSuperAdminClient(conn)
	saKey, err := superadmin.GenerateToken(time.Now().Add(24*time.Hour), "test-key")
	Expect(err).To(BeNil())
	getContext := func() context.Context {
		return metadata.AppendToOutgoingContext(context.Background(), "authorization", saKey)
	}

	Describe("Set/Get time", func() {
		BeforeEach(func() {
			cleanupMongo()
		})

		FSpecify("happy path", func() {
			start := int64(1635554584)
			_, err := superadminClient.SetTime(getContext(), &pb.SetTimeRequest{
				Start: time.Unix(start, 0).Format(time.RFC3339),
				End:   time.Unix(start+10000, 0).Format(time.RFC3339),
			})
			Expect(err).To(BeNil())

			res, err := superadminClient.GetTime(getContext(), &pb.GetTimeRequest{})
			Expect(err).To(BeNil())
			t1, err := time.Parse(time.RFC3339, res.Start)
			Expect(err).To(BeNil())
			Expect(t1.Unix()).To(Equal(start))
			t2, err := time.Parse(time.RFC3339, res.End)
			Expect(err).To(BeNil())
			Expect(t2.Unix()).To(Equal(start + 10000))
		})
	})
})
