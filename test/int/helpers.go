package int

import (
	"context"
	. "github.com/onsi/gomega"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/options"
)

func cleanupMongo() {
	m, err := mongo.Connect(context.Background(), options.Client().ApplyURI("mongodb://mongo1:27017,mongo2:27018,mongo3:27019/?replicaSet=rs0"))
	Expect(err).To(BeNil())
	db := m.Database("comp")

	collections := []string{"auth", "solutions", "problems", "time"}
	for _, v := range collections {
		_, err := db.Collection(v).DeleteMany(context.Background(), bson.M{})
		Expect(err).To(BeNil())
	}
}
