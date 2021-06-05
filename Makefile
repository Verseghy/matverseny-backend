.PHONY: proto
proto:
	protoc --proto_path=. --go_out=:. --go-grpc_out=. proto/auth.proto
	protoc --proto_path=. --go_out=:. --go-grpc_out=. proto/competition.proto
