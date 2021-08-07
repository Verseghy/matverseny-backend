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

	var rt1 string
	var at1 string
	var rt2 string
	_ = rt2
	var at2 string

	BeforeEach(func() {
		cleanupMongo()
		rt1, at1 = registerUser(authClient, 0)
		rt2, at2 = registerUser(authClient, 1)
	})

	Describe("Create Team", func() {
		Specify("happy path", func() {
			_, err := teamClient.CreateTeam(authenticatedContext(at1), &pb.CreateTeamRequest{
				Name: "Test team",
			})
			Expect(err).To(BeNil())

			_, ac := refreshJWT(authClient, rt1)
			Expect(ac.Team).NotTo(BeEmpty())
		})
		Specify("sad path - too long username", func() {
			_, err := teamClient.CreateTeam(authenticatedContext(at1), &pb.CreateTeamRequest{
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
		Specify("sad path - team name exists", func() {
			_, err := teamClient.CreateTeam(authenticatedContext(at1), &pb.CreateTeamRequest{
				Name: "test",
			})
			Expect(err).To(BeNil())

			_, err = teamClient.CreateTeam(authenticatedContext(at2), &pb.CreateTeamRequest{
				Name: "test",
			})
			Expect(err).NotTo(BeNil())
			Expect(err.Error()).To(ContainSubstring(errs.ErrTeamNameTaken.Error()))
		})
		Specify("sad path - user already in team", func() {
			_, err := teamClient.CreateTeam(authenticatedContext(at1), &pb.CreateTeamRequest{
				Name: "test",
			})
			Expect(err).To(BeNil())
			_, err = teamClient.CreateTeam(authenticatedContext(at1), &pb.CreateTeamRequest{
				Name: "test",
			})
			Expect(err).NotTo(BeNil())
			Expect(err.Error()).To(ContainSubstring(errs.ErrHasTeam.Error()))
		})
	})
})
