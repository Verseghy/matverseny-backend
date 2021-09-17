package int

import (
	"matverseny-backend/errs"
	pb "matverseny-backend/proto"

	. "github.com/onsi/ginkgo"
	. "github.com/onsi/gomega"
	"google.golang.org/grpc"
)

var _ = Describe("Team", func() {
	var opts []grpc.DialOption
	opts = append(opts, grpc.WithInsecure())

	conn, err := grpc.Dial("localhost:6969", opts...)
	Expect(err).To(BeNil())

	authClient := pb.NewAuthClient(conn)
	teamClient := pb.NewTeamClient(conn)

	var user1 User
	var user2 User
	var user3 User
	var user4 User
	
	BeforeEach(func() {
		cleanupMongo()
		user1 = registerUser(authClient, 0)
		user2 = registerUser(authClient, 1)
		user3 = registerUser(authClient, 2)
		user4 = registerUser(authClient, 3)
	})

	Describe("Create Team", func() {
		Specify("happy path", func() {
			_, err := teamClient.CreateTeam(user1.Context(), &pb.CreateTeamRequest{
				Name: "Test team",
			})
			Expect(err).To(BeNil())

			user1.Refresh()
			Expect(user1.Claims.Team).NotTo(BeEmpty())
		})
		Specify("sad path - too long username", func() {
			_, err := teamClient.CreateTeam(user1.Context(), &pb.CreateTeamRequest{
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
			_, err := teamClient.CreateTeam(user1.Context(), &pb.CreateTeamRequest{
				Name: "test",
			})
			Expect(err).To(BeNil())

			_, err = teamClient.CreateTeam(user2.Context(), &pb.CreateTeamRequest{
				Name: "test",
			})
			Expect(err).NotTo(BeNil())
			Expect(err.Error()).To(ContainSubstring(errs.ErrTeamNameTaken.Error()))
		})
		Specify("sad path - user already in team", func() {
			_, err := teamClient.CreateTeam(user1.Context(), &pb.CreateTeamRequest{
				Name: "test",
			})
			Expect(err).To(BeNil())
			_, err = teamClient.CreateTeam(user1.Context(), &pb.CreateTeamRequest{
				Name: "test",
			})
			Expect(err).NotTo(BeNil())
			Expect(err.Error()).To(ContainSubstring(errs.ErrHasTeam.Error()))
		})
	})

	Describe("Kick User", func() {
		BeforeEach(func() {
			_, err = teamClient.CreateTeam(user1.Context(), &pb.CreateTeamRequest{
				Name: "Test team",
			})

			Expect(err).To(BeNil())

			info, _ := teamClient.GetTeamInfo(user1.Context(), &pb.GetTeamInfoRequest{})

			Expect(info).NotTo(BeNil())
			Expect(info.JoinCode).NotTo(BeNil())
   
			teamClient.JoinTeam(user2.Context(), &pb.JoinTeamRequest{ Code: info.JoinCode })
			teamClient.JoinTeam(user3.Context(), &pb.JoinTeamRequest{ Code: info.JoinCode })
			teamClient.JoinTeam(user4.Context(), &pb.JoinTeamRequest{ Code: info.JoinCode })

			user1.Refresh()
			user2.Refresh()
			user3.Refresh()
			user4.Refresh()

			_, err = teamClient.ChangeCoOwnerStatus(user1.Context(), &pb.ChangeCoOwnerStatusRequest{
				UserId: user2.UserID(),
				ShouldCoowner: true,
			})

			Expect(err).To(BeNil())
		})

		Specify("Owner cannot kick himself", func() {
			_, err = teamClient.KickUser(user1.Context(), &pb.KickUserRequest{
				UserId: user1.UserID(),
			})
			Expect(err).NotTo(BeNil())
			Expect(err.Error()).To(ContainSubstring(errs.ErrNotAuthorized.Error()))
		})

		Specify("Owner can kick the co-owner", func() {
			_, err = teamClient.KickUser(user1.Context(), &pb.KickUserRequest{
				UserId: user2.UserID(),
			})
			Expect(err).To(BeNil())
		})

		Specify("Owner can kick a member", func() {
			_, err = teamClient.KickUser(user1.Context(), &pb.KickUserRequest{
				UserId: user3.UserID(),
			})
			Expect(err).To(BeNil())
		})

		Specify("Co-owner cannot kick the owner", func() {
			_, err = teamClient.KickUser(user2.Context(), &pb.KickUserRequest{
				UserId: user1.UserID(),
			})
			Expect(err).NotTo(BeNil())
			Expect(err.Error()).To(ContainSubstring(errs.ErrNotAuthorized.Error()))
		})

		Specify("Co-owner cannot kick himself", func() {
			_, err = teamClient.KickUser(user2.Context(), &pb.KickUserRequest{
				UserId: user2.UserID(),
			})
			Expect(err).NotTo(BeNil())
			Expect(err.Error()).To(ContainSubstring(errs.ErrNotAuthorized.Error()))
		})

		Specify("Co-owner can kick a member", func() {
			_, err = teamClient.KickUser(user2.Context(), &pb.KickUserRequest{
				UserId: user3.UserID(),
			})
			Expect(err).To(BeNil())
		})

		Specify("Member cannot kick the Owner", func() {
			_, err = teamClient.KickUser(user3.Context(), &pb.KickUserRequest{
				UserId: user1.UserID(),
			})
			Expect(err).NotTo(BeNil())
			Expect(err.Error()).To(ContainSubstring(errs.ErrNotAuthorized.Error()))
		})

		Specify("Member cannot kick the co-owner", func() {
			_, err = teamClient.KickUser(user3.Context(), &pb.KickUserRequest{
				UserId: user2.UserID(),
			})
			Expect(err).NotTo(BeNil())
			Expect(err.Error()).To(ContainSubstring(errs.ErrNotAuthorized.Error()))
		})

		Specify("Member cannot kick a member", func() {
			_, err = teamClient.KickUser(user3.Context(), &pb.KickUserRequest{
				UserId: user4.UserID(),
			})
			Expect(err).NotTo(BeNil())
			Expect(err.Error()).To(ContainSubstring(errs.ErrNotAuthorized.Error()))
		})

		Specify("Member cannot kick himself", func() {
			_, err = teamClient.KickUser(user3.Context(), &pb.KickUserRequest{
				UserId: user3.UserID(),
			})
			Expect(err).NotTo(BeNil())
			Expect(err.Error()).To(ContainSubstring(errs.ErrNotAuthorized.Error()))
		})

		Specify("Co-owner has rank after kicking", func() {
			_, err = teamClient.KickUser(user2.Context(), &pb.KickUserRequest{
				UserId: user3.UserID(),
			})
			Expect(err).To(BeNil())

			info, err := teamClient.GetTeamInfo(user2.Context(), &pb.GetTeamInfoRequest{})
			Expect(err).To(BeNil())
			Expect(info).NotTo(BeNil())
			Expect(info.Members).NotTo(BeNil())
			Expect(info.Members).NotTo(BeEmpty())
			Expect(info.Members).To(ContainElement(&pb.GetTeamInfoResponse_Member{
				ID: user2.UserID(),
				Name: "test",
				Class: 0,
				Rank: pb.GetTeamInfoResponse_Member_k_COOWNER,
			}))
		})
	})
})
