curl -X POST \
    -H "Content-Type: application/json" \
    -d '{
        "sender": "0xc7d158a9b05c6dfd07c233649c9e0f78320d066e5ac0e8f5101de500ad9e84e8",
        "content_type": "Music"        
    }' \
    http://localhost:3001/sign_and_fund_transaction

# Change the sender address to the one you want to use