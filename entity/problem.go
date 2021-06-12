package entity

import (
	"go.mongodb.org/mongo-driver/bson/primitive"
	pb "matverseny-backend/proto"
)

type Problem struct {
	ID       primitive.ObjectID `bson:"id"`
	Body     string             `bson:"body"`
	Image    string             `bson:"image"`
	Position uint32             `bson:"position"`
	Solution int64              `bson:"solution"`
}

func (p *Problem) ToProto() *pb.Problem {
	return &pb.Problem{
		Id:    p.ID.Hex(),
		Body:  p.Body,
		Image: p.Image,
	}
}

func (p *Problem) ToAdminProto() *pb.Problem {
	return &pb.Problem{
		Id:       p.ID.Hex(),
		Body:     p.Body,
		Image:    p.Image,
		Solution: p.Solution,
	}
}

func (p *Problem) FromProto(proto *pb.Problem) {
	p.Body = proto.Body
	p.Image = proto.Image
	p.Solution = proto.Solution
}
