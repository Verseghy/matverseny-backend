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

	Describe("Create Team", func() {
		var user1 User
		var user2 User

		BeforeEach(func() {
			cleanupMongo()
			user1 = registerUser(authClient, 0)
			user2 = registerUser(authClient, 1)
		})

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
			Expect(err).To(MatchBackendError(errs.ErrTeamNameTooLong))
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
			Expect(err).To(MatchBackendError(errs.ErrTeamNameTaken))
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
			Expect(err).To(MatchBackendError(errs.ErrHasTeam))
		})
	})

	Describe("Join Team", func() {
		var user1 User
		var user2 User
		var user3 User

		var team1 Team
		var team2 Team

		BeforeEach(func() {
			cleanupMongo()
			user1 = registerUser(authClient, 0)
			user2 = registerUser(authClient, 1)
			user3 = registerUser(authClient, 2)

			team1 = createTeam(user1, "Test team 1", teamClient)
			team2 = createTeam(user2, "Test team 2", teamClient)
		})

		Specify("Should join team", func() {
			_, err := teamClient.JoinTeam(user3.Context(), &pb.JoinTeamRequest{
				Code: team1.JoinCode,
			})

			Expect(err).To(BeNil())
		})

		Specify("Can't join team twice", func() {
			_, err := teamClient.JoinTeam(user3.Context(), &pb.JoinTeamRequest{
				Code: team1.JoinCode,
			})

			Expect(err).To(BeNil())

			_, err = teamClient.JoinTeam(user3.Context(), &pb.JoinTeamRequest{
				Code: team1.JoinCode,
			})

			Expect(err).To(MatchBackendError(errs.ErrHasTeam))
		})

		Specify("Can't join to another team", func() {
			_, err := teamClient.JoinTeam(user3.Context(), &pb.JoinTeamRequest{
				Code: team1.JoinCode,
			})

			Expect(err).To(BeNil())

			_, err = teamClient.JoinTeam(user3.Context(), &pb.JoinTeamRequest{
				Code: team2.JoinCode,
			})

			Expect(err).To(MatchBackendError(errs.ErrHasTeam))
		})

		Specify("Wrong join code", func() {
			_, err := teamClient.JoinTeam(user3.Context(), &pb.JoinTeamRequest{
				Code: "0",
			})

			Expect(err).To(MatchBackendError(errs.ErrNotFound))
		})
	})

	Describe("Leave Team", func() {
		var user1 User
		var user2 User

		var team Team

		BeforeEach(func() {
			cleanupMongo()
			user1 = registerUser(authClient, 0)
			user2 = registerUser(authClient, 1)

			team = createTeam(user1, "Test team 1", teamClient)
		})

		Specify("Can't leave without a team", func() {
			_, err := teamClient.LeaveTeam(user2.Context(), &pb.LeaveTeamRequest{})

			Expect(err).To(MatchBackendError(errs.ErrNoTeam))
		})

		Specify("Owner can't leave", func() {
			_, err := teamClient.LeaveTeam(user1.Context(), &pb.LeaveTeamRequest{})

			Expect(err).To(MatchBackendError(errs.ErrOwnerCantLeave))
		})

		Specify("Member can leave", func() {
			team.AddMember(user2, false)
			_, err := teamClient.LeaveTeam(user2.Context(), &pb.LeaveTeamRequest{})

			Expect(err).To(BeNil())
		})

		Specify("Coowner can leave", func() {
			team.AddMember(user2, true)
			_, err := teamClient.LeaveTeam(user2.Context(), &pb.LeaveTeamRequest{})

			Expect(err).To(BeNil())
		})
	})

	Describe("Get Team Info", func() {
		var user1 User
		var user2 User

		var team Team

		BeforeEach(func() {
			cleanupMongo()
			user1 = registerUser(authClient, 0)
			user2 = registerUser(authClient, 1)

			team = createTeam(user1, "Test Team", teamClient)
		})

		Specify("Can't get team info without a team", func() {
			info, err := teamClient.GetTeamInfo(user2.Context(), &pb.GetTeamInfoRequest{})
			Expect(err).To(MatchBackendError(errs.ErrNoTeam))
			Expect(info).To(BeNil())
		})

		Specify("Member list changes", func() {
			info, err := teamClient.GetTeamInfo(user1.Context(), &pb.GetTeamInfoRequest{})
			Expect(err).To(BeNil())
			Expect(info).NotTo(BeNil())
			Expect(info.Members).NotTo(BeNil())
			Expect(info.Members).To(HaveLen(1))

			team.AddMember(user2, false)

			info, err = teamClient.GetTeamInfo(user1.Context(), &pb.GetTeamInfoRequest{})
			Expect(err).To(BeNil())
			Expect(info).NotTo(BeNil())
			Expect(info.Members).NotTo(BeNil())
			Expect(info.Members).To(HaveLen(2))
		})
	})

	Describe("Update Team", func() {
		var user1 User
		var user2 User

		var team Team

		BeforeEach(func() {
			cleanupMongo()
			user1 = registerUser(authClient, 0)
			user2 = registerUser(authClient, 1)

			team = createTeam(user1, "Test Team", teamClient)
		})

		Specify("Change team name", func() {
			By("Get initial name")
			info, err := teamClient.GetTeamInfo(user1.Context(), &pb.GetTeamInfoRequest{})

			Expect(err).To(BeNil())
			Expect(info).NotTo(BeNil())
			Expect(info.Name).To(Equal("Test Team"))

			By("Update name")
			_, err = teamClient.UpdateTeam(user1.Context(), &pb.UpdateTeamRequest{
				Name: "New test name",
			})

			Expect(err).To(BeNil())

			By("Get new name")
			info, err = teamClient.GetTeamInfo(user1.Context(), &pb.GetTeamInfoRequest{})

			Expect(err).To(BeNil())
			Expect(info).NotTo(BeNil())
			Expect(info.Name).To(Equal("New test name"))
		})

		Specify("No team", func() {
			_, err = teamClient.UpdateTeam(user2.Context(), &pb.UpdateTeamRequest{
				Name: "New test name",
			})
			Expect(err).To(MatchBackendError(errs.ErrNoTeam))
		})

		Specify("Member cannot change name", func() {
			team.AddMember(user2, false)
			_, err = teamClient.UpdateTeam(user2.Context(), &pb.UpdateTeamRequest{
				Name: "New test name",
			})
			Expect(err).To(MatchBackendError(errs.ErrNotAuthorized))
		})

		Specify("Co-owner cannot change name", func() {
			team.AddMember(user2, true)
			_, err = teamClient.UpdateTeam(user2.Context(), &pb.UpdateTeamRequest{
				Name: "New test name",
			})
			Expect(err).To(MatchBackendError(errs.ErrNotAuthorized))
		})

		Specify("Team name already exists", func() {
			createTeam(user2, "Test Team 2", teamClient)

			_, err = teamClient.UpdateTeam(user1.Context(), &pb.UpdateTeamRequest{
				Name: "Test Team 2",
			})
			Expect(err).To(MatchBackendError(errs.ErrTeamNameTaken))
		})
	})

	Describe("Kick User", func() {
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

			team := createTeam(user1, "Test team", teamClient)
			team.AddMember(user2, true)
			team.AddMember(user3, false)
			team.AddMember(user4, false)
		})

		Specify("Owner cannot kick himself", func() {
			_, err = teamClient.KickUser(user1.Context(), &pb.KickUserRequest{
				UserId: user1.UserID(),
			})
			Expect(err).NotTo(BeNil())
			Expect(err).To(MatchBackendError(errs.ErrNotAuthorized))
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
			Expect(err).To(MatchBackendError(errs.ErrNotAuthorized))
		})

		Specify("Co-owner cannot kick himself", func() {
			_, err = teamClient.KickUser(user2.Context(), &pb.KickUserRequest{
				UserId: user2.UserID(),
			})
			Expect(err).NotTo(BeNil())
			Expect(err).To(MatchBackendError(errs.ErrNotAuthorized))
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
			Expect(err).To(MatchBackendError(errs.ErrNotAuthorized))
		})

		Specify("Member cannot kick the co-owner", func() {
			_, err = teamClient.KickUser(user3.Context(), &pb.KickUserRequest{
				UserId: user2.UserID(),
			})
			Expect(err).NotTo(BeNil())
			Expect(err).To(MatchBackendError(errs.ErrNotAuthorized))
		})

		Specify("Member cannot kick a member", func() {
			_, err = teamClient.KickUser(user3.Context(), &pb.KickUserRequest{
				UserId: user4.UserID(),
			})
			Expect(err).NotTo(BeNil())
			Expect(err).To(MatchBackendError(errs.ErrNotAuthorized))
		})

		Specify("Member cannot kick himself", func() {
			_, err = teamClient.KickUser(user3.Context(), &pb.KickUserRequest{
				UserId: user3.UserID(),
			})
			Expect(err).NotTo(BeNil())
			Expect(err).To(MatchBackendError(errs.ErrNotAuthorized))
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

	Describe("Generate Join Code", func() {
		var user1 User
		var user2 User
		var user3 User

		BeforeEach(func() {
			cleanupMongo()
			user1 = registerUser(authClient, 0)
			user2 = registerUser(authClient, 1)
			user3 = registerUser(authClient, 2)
		})

		Context("Has team", func() {
			BeforeEach(func() {
				team := createTeam(user1, "Test team", teamClient)
				team.AddMember(user2, true)
				team.AddMember(user3, false)
			})

			Specify("Owner can generate join code", func() {
				res, err := teamClient.GenerateJoinCode(user1.Context(), &pb.GenerateJoinCodeRequest{})

				Expect(err).To(BeNil())
				Expect(res).NotTo(BeNil())
				Expect(res.NewCode).NotTo(BeNil())
			})

			Specify("Co-owner can't generate join code", func() {
				res, err := teamClient.GenerateJoinCode(user2.Context(), &pb.GenerateJoinCodeRequest{})

				Expect(res).To(BeNil())
				Expect(err).To(MatchBackendError(errs.ErrNotAuthorized))
			})

			Specify("Member can't generate join code", func() {
				res, err := teamClient.GenerateJoinCode(user3.Context(), &pb.GenerateJoinCodeRequest{})

				Expect(res).To(BeNil())
				Expect(err).To(MatchBackendError(errs.ErrNotAuthorized))
			})
		})


		Context("No team", func() {
			Specify("Cannot generate join code without a team", func() {
				res, err := teamClient.GenerateJoinCode(user1.Context(), &pb.GenerateJoinCodeRequest{})

				Expect(res).To(BeNil())
				Expect(err).To(MatchBackendError(errs.ErrNoTeam))
			})
		})
	})
})
