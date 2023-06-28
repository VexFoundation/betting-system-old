# betting-system-1
This is the smart contract and frontend for the betting system for the VEX project. It runs on the testnet and has no DAO, oracle or FT associated with it.

<br />

# Quickstart

1. Make sure you have installed [rust](https://rust.org/).
2. Install the [`NEAR CLI`](https://github.com/near/near-cli#setup)

<br />

## 1. Build and Deploy the Contract
You can automatically compile and deploy the contract in the NEAR testnet by running (inside of contract):

```bash
./deploy.sh
```

You may need to change the permissions on the deploy.sh and build.sh file, do this by running:

```bash
chmod +x deploy.sh
chmod +x build.sh
```

Once finished, check the `neardev/dev-account` file to find the address in which the contract was deployed:

```bash
cat ./neardev/dev-account
# e.g. dev-1659899566943-21539992274727
```

<br />

The contract can be used via the CLI with the following commands: 

```bash
near call <dev account name> create_match '{"team_1": " ", "team_2": " ", "in_odds_1": " ", "in_odds_2": " ", "date": " "}' --accountId <dev account name>
near call <dev account name> finish_match '{"match_id": " ", "winning_team": " "}' --accountId <dev account name>
near call <dev account name> make_bet '{"match_id": " ", "decision": " "}' --amount 2 --accountId <your account name>
near view <dev account name> view_matches '{"match_id": " "}'
near view <dev account name> view_bets '{"match_id": " ", "name": " "}'
near view <dev account name> view_potential_winnings '{"match_id": " ", "team": " ", "bet_amount": " "}'
```

The contract can be interacted with using the front end via the command:

```bash
npm start
```
