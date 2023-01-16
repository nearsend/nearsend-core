const { keyStores, connect } = require('near-api-js');
const path = require("path");
const homedir = require("os").homedir();


const env = {
    WASM_PATH: "./target/res/bulk_sender.wasm",
    testnet: {
        CONTRACT_ACCOUNT_ID: "nearsend.testnet",
        ORACLE_ACCOUNT_ID: "fpo.opfilabs.testnet",
        ORACLE_PROVIDER_ID: "opfilabs.testnet",
    },
    mainnet: {
        CONTRACT_ACCOUNT_ID: "bulksender.near",
        ORACLE_ACCOUNT_ID: "fpo.opfilabs.near",
        ORACLE_PROVIDER_ID: "opfilabs.near",
    }
};

const setUpTestnet = async () => {
    const CREDENTIALS_DIR = ".near-credentials";
    const credentialsPath = path.join(homedir, CREDENTIALS_DIR);
    const keyStore = new keyStores.UnencryptedFileSystemKeyStore(credentialsPath);

    const config = {
        keyStore,
        networkId: "testnet",
        nodeUrl: "https://rpc.testnet.near.org",
    };
    return near = await connect({ ...config, keyStore });
};

const setUpMainnet = async () => {
    const CREDENTIALS_DIR = ".near-credentials";
    const credentialsPath = path.join(homedir, CREDENTIALS_DIR);
    const keyStore = new keyStores.UnencryptedFileSystemKeyStore(credentialsPath);

    const config = {
        keyStore,
        networkId: "mainnet",
        nodeUrl: "https://rpc.mainnet.near.org",
    };
    return near = await connect({ ...config, keyStore });
};

module.exports = {
    env,
    setUpTestnet,
    setUpMainnet,
};