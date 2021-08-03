package int

import (
	. "github.com/onsi/ginkgo"
	. "github.com/onsi/gomega"
	"google.golang.org/grpc"
	"matverseny-backend/errs"
	pb "matverseny-backend/proto"
)

var _ = Describe("Team", func() {
	var opts []grpc.DialOption
	opts = append(opts, grpc.WithInsecure())

	conn, err := grpc.Dial("localhost:6969", opts...)
	Expect(err).To(BeNil())

	authClient := pb.NewAuthClient(conn)
	teamClient := pb.NewTeamClient(conn)

	var rt string
	var at string

	BeforeEach(func() {
		cleanupMongo()
		rt, at = registerUser(authClient)
	})

	Describe("Create Team", func() {
		Specify("happy path", func() {
			_, err := teamClient.CreateTeam(authenticatedContext(at), &pb.CreateTeamRequest{
				Name: "Test team",
			})
			Expect(err).To(BeNil())

			_, ac := refreshJWT(authClient, rt)
			Expect(ac.Team).NotTo(BeEmpty())
		})

		Specify("sad path - too long username", func() {
			_, err := teamClient.CreateTeam(authenticatedContext(at), &pb.CreateTeamRequest{
				Name: func() string {
					var s string
					for i := 0; i < 67; i++ {
						s += "A"
					}
					return s
				}(),
			})
			Expect(err).NotTo(BeNil())
			Expect(err.Error()).To(ContainSubstring(errs.ErrTeamNameTooLong.Error()))
		})
	})
})
