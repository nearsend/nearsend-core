const { transactions } = require('near-api-js');
const { env, setUpTestnet } = require('./config.js');
const fs = require("fs");
const shell = require('shelljs');

(async () => {

    if (shell.exec('bash shell-script/build.sh').code !== 0) {
        shell.echo('Error: Failed to build smart contract');
        shell.exit(1);
    }

    const near = await setUpTestnet();

    sendTransactions();

    async function sendTransactions() {
        const account = await near.account(env.testnet.CONTRACT_ACCOUNT_ID);
        const result = await account.signAndSendTransaction({
            receiverId: env.testnet.CONTRACT_ACCOUNT_ID,
            actions: [
                transactions.deployContract(fs.readFileSync(env.WASM_PATH)),
                transactions.functionCall(
                    "migrate",
                    Buffer.from(JSON.stringify({})),
                    200000000000000,
                    // "1"
                ),
                transactions.functionCall(
                    "set_oracle",
                    Buffer.from(JSON.stringify({
                        "oracle_account_id": env.testnet.ORACLE_ACCOUNT_ID,
                        "oracle_provider_id": env.testnet.ORACLE_PROVIDER_ID,
                    })),
                    30000000000000,
                ),
            ],
        });

        console.log(result.receipts_outcome);
        result.receipts_outcome.map(e => console.log(e.outcome));
    }
})();