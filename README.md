# [iLend.xyz](https://ilend.xyz/)

```                                                                                          
                                                                                  dddddddd
  iiii  LLLLLLLLLLL                                                               d::::::d
 i::::i L:::::::::L                                                               d::::::d
  iiii  L:::::::::L                                                               d::::::d
        LL:::::::LL                                                               d:::::d 
iiiiiii   L:::::L                   eeeeeeeeeeee    nnnn  nnnnnnnn        ddddddddd:::::d 
i:::::i   L:::::L                 ee::::::::::::ee  n:::nn::::::::nn    dd::::::::::::::d 
 i::::i   L:::::L                e::::::eeeee:::::een::::::::::::::nn  d::::::::::::::::d 
 i::::i   L:::::L               e::::::e     e:::::enn:::::::::::::::nd:::::::ddddd:::::d 
 i::::i   L:::::L               e:::::::eeeee::::::e  n:::::nnnn:::::nd::::::d    d:::::d 
 i::::i   L:::::L               e:::::::::::::::::e   n::::n    n::::nd:::::d     d:::::d 
 i::::i   L:::::L               e::::::eeeeeeeeeee    n::::n    n::::nd:::::d     d:::::d 
 i::::i   L:::::L         LLLLLLe:::::::e             n::::n    n::::nd:::::d     d:::::d 
i::::::iLL:::::::LLLLLLLLL:::::Le::::::::e            n::::n    n::::nd::::::ddddd::::::dd
i::::::iL::::::::::::::::::::::L e::::::::eeeeeeee    n::::n    n::::n d:::::::::::::::::d
i::::::iL::::::::::::::::::::::L  ee:::::::::::::e    n::::n    n::::n  d:::::::::ddd::::d
iiiiiiiiLLLLLLLLLLLLLLLLLLLLLLLL    eeeeeeeeeeeeee    nnnnnn    nnnnnn   ddddddddd   ddddd
                                                                       
```

 iLend Protocol
=================


The iLend Protocol is an Injective smart contract for supplying or borrowing assets.  It offers the ability to engage in lending and borrowing activities in a decentralized, transparent, and efficient manner. The protocol leverages Injective Protocol's ability to offer fast, secure, and EVM-compatible DeFi transactions across multiple blockchain ecosystems


Contracts
=================

Current version of the contract address `inj1xjkfkfgjg60gh3duf5hyk3vfsluyurjljznwgu` deployed at https://explorer.injective.network/contract/inj1xjkfkfgjg60gh3duf5hyk3vfsluyurjljznwgu/

There is a single core contract operating as iLend Protocol

### Lending Contract
Which are self-contained borrowing and lending contract. Lending Contract consists of Markets (also can be referred as  Supported Tokens).
Each Market is assigned an interest rate and risk model, and allows accounts to *mint* (supply capital), *redeem* (withdraw capital), *borrow* and *repay a borrow*.


## Installation

To run the project you need:

1. ```
    git clone https://github.com/i-Lend-org/iLend-smart-contracts.git
    cd iLend-smart-contracts
    cargo build --release && cargo test 
 2. For Developers or Advanced Users: [injectived](https://docs.injective.network/develop/guides/cosmwasm-dapps/Your_first_contract_on_injective#install-injectived) command-line interface. This enables to interact with the Injective blockchain within CLI 


Developers
=================
1. Having [injectived](https://docs.injective.network/develop/guides/cosmwasm-dapps/Your_first_contract_on_injective#install-injectived) install you can interact with iLend Smart Contract from the CLI
2. See Injectived Docs at [injectived/welcome](https://docs.injective.network/develop/tools/injectived/welcome)
3. Make any transaction adhering  Execute/Query Msgs [here](schema)

Documentation
=================

To see full detailed Documentation and Roadmap visit the [iLend Docs](https://docs.ilend.xyz/ilend-knowledge-hub/)

Discussion
=================

Both for community or security concerns  you can use the most convenient way to contact us:

- [X / Twitter](https://twitter.com/ilendorg)
- [Github](https://github.com/i-Lend-org/iLend-smart-contracts)






© Copyright 2024, Zpoken Team
