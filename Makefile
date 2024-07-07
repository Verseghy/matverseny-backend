all:

.PHONY: build-images
build-images:
	podman build -t ghcr.io/verseghy/matverseny-setup -f containerfiles/setup.Containerfile .
	podman build -t ghcr.io/verseghy/matverseny-migration -f containerfiles/migration.Containerfile .
	podman build -t ghcr.io/verseghy/matverseny-backend .
