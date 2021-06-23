package entity

import "go.mongodb.org/mongo-driver/bson/primitive"

type Team struct {
	ID       primitive.ObjectID   `bson:"_id,omitempty"`
	TeamName string               `bson:"team_name"`
	Members  []primitive.ObjectID `bson:"members"`
	Owner    primitive.ObjectID   `bson:"owner"`
	CoOwner  *primitive.ObjectID  `bson:"co_owner,omitempty"`
	Locked   bool                 `bson:"locked"`
	JoinCode string               `bson:"join_code"`
}
