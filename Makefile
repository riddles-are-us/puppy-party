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

run:
	node ./ts/src/service.js

deploy:
	docker build --file ./deploy/service.docker -t zkwasm-server . --network=host
