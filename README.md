# General Set
Set the card game with generalized features

## Install
```
npm install -g typescript
npm install -g nodemon
npm install -g concurrently

// install yarn
yarn install
/* cd client && yarn install */
/* cd server && yarn install */

yarn build
yarn start
```

## Deploy
- `cd client && yarn build`
- `cd server && chmod +x ./app.js && pm2 kill && pm2 start app.js`
