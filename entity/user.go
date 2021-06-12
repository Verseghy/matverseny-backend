package entity

import "go.mongodb.org/mongo-driver/bson/primitive"

type User struct {
	ID       primitive.ObjectID `bson:"_id"`
	Email    string             `bson:"email"`
	Password string             `bson:"password"`
	Name     string             `bson:"name"`
	School   string             `bson:"school"`
	Class    uint32             `bson:"class"`

	IsAdmin bool   `bson:"is_admin"`
	Team    string `bson:"team"`
}
