package int

import (
	"context"
	"github.com/golang-jwt/jwt/v4"
	. "github.com/onsi/ginkgo"
	. "github.com/onsi/gomega"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/options"
	"google.golang.org/grpc/metadata"
	jwt2 "matverseny-backend/jwt"
	pb "matverseny-backend/proto"
	"strconv"
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

type User struct {
	AccessToken string
	RefreshToken string
	Claims *jwt2.AccessClaims
	RefreshClaims *jwt2.RefreshClaims
	authClient pb.AuthClient
}

func (user *User) Context() context.Context {
	return metadata.AppendToOutgoingContext(context.Background(), "authorization", user.AccessToken)
}

func (user *User) UserID() string {
	return user.Claims.UserID
}

func (user *User) Refresh() {
	res, err := user.authClient.RefreshToken(context.Background(), &pb.RefreshTokenRequest{
		Token: user.RefreshToken,
	})

	Expect(err).To(BeNil())
	Expect(res.Token).NotTo(BeNil())

	user.AccessToken = res.Token
	user.ParseTokens()
}

func (user *User) ParseTokens() {
	rt, err := jwt.ParseWithClaims(user.RefreshToken, &jwt2.RefreshClaims{}, func(token *jwt.Token) (interface{}, error) {
		return key, nil
	})
	Expect(err).To(BeNil())
	c, ok := rt.Claims.(*jwt2.RefreshClaims)
	Expect(ok).To(BeTrue())
	Expect(c.ExpiresAt).To(Satisfy(func(t int64) bool { return time.Now().Unix() < t }))
	Expect(c.UserID).NotTo(BeEmpty())

	at, err := jwt.ParseWithClaims(user.AccessToken, &jwt2.AccessClaims{}, func(token *jwt.Token) (interface{}, error) {
		return key, nil
	})
	Expect(err).To(BeNil())
	c2, ok := at.Claims.(*jwt2.AccessClaims)
	Expect(ok).To(BeTrue())
	Expect(c2.ExpiresAt).To(Satisfy(func(t int64) bool { return time.Now().Unix() < t }))
	Expect(c2.UserID).NotTo(BeEmpty())

	user.RefreshClaims = c
	user.Claims = c2
}

func registerUser(authClient pb.AuthClient, uid int) (user User) {
	res, err := authClient.Register(context.Background(), &pb.RegisterRequest{
		Email:    "test@test.test" + strconv.Itoa(uid),
		Password: "testtest",
		Name:     "test",
		School:   "test",
		Class:    0,
	})
	Expect(err).To(BeNil())

	Expect(res.RefreshToken).NotTo(BeNil())
	Expect(res.AccessToken).NotTo(BeNil())

	user.RefreshToken = res.RefreshToken
	user.AccessToken = res.AccessToken
	user.authClient = authClient

	user.ParseTokens()

	return
}

type Team struct {
	teamClient pb.TeamClient

	JoinCode string
	Owner User
} 

func createTeam(owner User, name string, teamClient pb.TeamClient) (team Team) {
	By("Create Team")

	_, err := teamClient.CreateTeam(owner.Context(), &pb.CreateTeamRequest{
		Name: name,
	})

	Expect(err).To(BeNil())

	info, err := teamClient.GetTeamInfo(owner.Context(), &pb.GetTeamInfoRequest{})

	Expect(err).To(BeNil())
	Expect(info).NotTo(BeNil())
	Expect(info.JoinCode).NotTo(BeNil())
	Expect(info.Name).To(Equal(name))
	Expect(len(info.Members)).To(Equal(1))

	owner.Refresh()

	team.teamClient = teamClient
	team.JoinCode = info.JoinCode
	team.Owner = owner

	return
}

func (team *Team) AddMember(user User, shouldCoowner bool) {
	By("Add Member to Team")

	_, err := team.teamClient.JoinTeam(user.Context(), &pb.JoinTeamRequest{
		Code: team.JoinCode,
	})

	Expect(err).To(BeNil())


	if (shouldCoowner == true) {
		By("Add Member - Set Rank")

		_, err := team.teamClient.ChangeCoOwnerStatus(team.Owner.Context(), &pb.ChangeCoOwnerStatusRequest{
			UserId: user.UserID(),
			ShouldCoowner: true,
		})

		Expect(err).To(BeNil())
	}

	user.Refresh()
}
