# Project Name

TODO: Write a project description

## Prerequisites

check the corresponding  [#prerequisites](https://docs.injective.network/develop/guides/cosmwasm-dapps/Your_first_contract_on_injective#prerequisites
)

## Unit tests

```
cargo unit-test // run this with RUST_BACKTRACE=1 for helpful backtraces
```

## Building the contract

check the corresponding  [#building-the-contract](https://docs.injective.network/develop/guides/cosmwasm-dapps/Your_first_contract_on_injective/#building-the-contract
)

```
cargo wasm
```

Next, we must optimize the contract in order to ready the code for upload to the chain. 

You have to run docker image of rust-optimizer (linux version) and he will compress wasm file
and create ./artifacts folder to store the result of execution

command for linux:

```
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.12
```

## To interact with an injective blockchain
you have to install correspondent CLI - I have problems for mac m1, so i suggest you won't have problems with linux docker image [see](https://docs.injective.network/develop/guides/cosmwasm-dapps/Your_first_contract_on_injective/#install-injectived)

once you install injective CLI, you have to create user and [airdrop yourself](
https://testnet.faucet.injective.network/) some tokens (this operation is allowed once a 24 hour)

## Upload the Wasm Contract

all the documentaion is needed here, as soon as you also have to be able to deploy that i suggest you to do it on your own following

https://docs.injective.network/develop/guides/cosmwasm-dapps/Your_first_contract_on_injective/#upload-the-wasm-contract

the main differences you might face:
1. deploying the contract != instantiating a contract - it is a separate action
2. your user address is not the same as your contract address, make sure you saved both
3. as for scripts you can use mine, but make sure you paste right addresses and CODE_ID 