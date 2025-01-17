INSTALL_DIR=./ts/node_modules/zkwasm-ts-server/src/application
RUNNING_DIR=./ts/node_modules/zkwasm-ts-server

.PHONY: deploy

default: build

./src/admin.pubkey: ./ts/node_modules/zkwasm-ts-server/src/init_admin.js
	node ./ts/node_modules/zkwasm-ts-server/src/init_admin.js ./src/admin.pubkey

./ts/src/service.js:
	cd ./ts && npx tsc && cd -

build: ./src/admin.pubkey
	wasm-pack build --release --out-name application --out-dir pkg
	#wasm-opt -Oz -o $(INSTALL_DIR)/application_bg.wasm pkg/application_bg.wasm
	cp pkg/application_bg.wasm $(INSTALL_DIR)/application_bg.wasm
	cp pkg/application_bg.js $(INSTALL_DIR)/application_bg.js
	cp pkg/application_bg.wasm.d.ts $(INSTALL_DIR)/application_bg.wasm.d.ts
	cd $(RUNNING_DIR) && npx tsc && cd -

clean:
	rm -rf pkg
	rm -rf ./src/admin.pubkey

run-zkwasm:
	node ./ts/src/service.js

run-express:
	node ./ts/src/express_server.js

run:
	@echo "Starting both servers..."
	@make -j2 run-zkwasm run-express


deploy:
	docker build --file ./deploy/service.docker -t zkwasm-server . --network=host
