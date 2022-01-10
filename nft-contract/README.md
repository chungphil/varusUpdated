# TBD
# TBD

### Set variable for contract address
source neardev/dev-account.env

### Initialise newly deployed contract
near call $CONTRACT_NAME new_default_meta '{"owner_id": "'$CONTRACT_NAME'"}' --accountId $CONTRACT_NAME

### Mint an NFT
near call $CONTRACT_NAME nft_mint '{"token_id": "0", "receiver_id": "'$CONTRACT_NAME'", "metadata": { "title": "thevarus", "description": "Pathogen", "media": "https://media.tenor.com/images/2f607eb7756f081d582b9a19907561ab/raw", "copies": 1}}' --accountId $CONTRACT_NAME --deposit 1

near call $con nft_mint '{"token_id": "0", "receiver_id": "'isparx.testnet'", "metadata": { "title": "Olympus Mons", "description": "Tallest mountain in charted solar system", "media": "https://upload.wikimedia.org/wikipedia/commons/thumb/0/00/Olympus_Mons_alt.jpg/1024px-Olympus_Mons_alt.jpg", "copies": 1}}' --accountId $con --deposit 1

### Enumeration calls

near call $CONTRACT_NAME nft_tokens --accountId $CONTRACT_NAME

near call $CONTRACT_NAME nft_tokens_for_owner '{"account_id":"'$CONTRACT_NAME'"}' --accountId $CONTRACT_NAME