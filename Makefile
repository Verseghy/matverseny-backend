.PHONY: proto
proto:
	cd ./proto/ && $(MAKE)

.PHONY: docker
docker:
	docker compose up --build --remove-orphans
