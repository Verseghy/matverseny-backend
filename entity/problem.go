package entity

import "go.mongodb.org/mongo-driver/bson/primitive"

type Problem struct {
	ID       primitive.ObjectID `bson:"id"`
	Body     string             `bson:"body"`
	Image    string             `bson:"image"`
	Position uint32             `bson:"position"`
}
