.PHONY: proto
proto:
	cd ./proto/ && $(MAKE)

.PHONY: docker
docker:
	docker-compose up --build --remove-orphans

.PHONY: docker-d
docker-d:
	docker-compose up --build --remove-orphans -d

.PHONY: int-run
int-run:
	go test ./test/int/... -v

.PHONY: superadmin-keygen
superadmin-keygen:
	go run cmd/superadmin-keygen/main.go $(ARGS)
