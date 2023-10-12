
import { createHelia } from 'helia'
import { mfs } from '@helia/mfs'
// import { IDBBlockstore } from 'blockstore-idb'
import { MemoryBlockstore } from 'blockstore-core'
import { MemoryDatastore } from 'datastore-core'

// create a blockstore - IndexedDB
// const blockstore = new IDBBlockstore("/ipfs-js");

// create a Helia node
import { createLibp2p } from 'libp2p'
import { noise } from '@chainsafe/libp2p-noise'
import { yamux } from '@chainsafe/libp2p-yamux'
import { bootstrap } from '@libp2p/bootstrap'
import { tcp } from '@libp2p/tcp'
import { identifyService } from 'libp2p/identify'

// TODO: use persistent blockstore and datastore

const datastore = new MemoryDatastore()
// libp2p is the networking layer that underpins Helia
const libp2p = await createLibp2p({
    datastore,
    addresses: {
      listen: [
        '/ip4/127.0.0.1/tcp/0'
      ]
    },
    transports: [
      tcp()
    ],
    connectionEncryption: [
      noise()
    ],
    streamMuxers: [
      yamux()
    ],
    peerDiscovery: [
      bootstrap({
        list: [
          '/dnsaddr/bootstrap.libp2p.io/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN',
          '/dnsaddr/bootstrap.libp2p.io/p2p/QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa',
          '/dnsaddr/bootstrap.libp2p.io/p2p/QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb',
          '/dnsaddr/bootstrap.libp2p.io/p2p/QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt'
        ]
      })
    ],
    services: {
      identify: identifyService()
    }
})

// create a Helia node
const helia = await createHelia({
    datastore,
    // datastore: new MemoryDatastore(),
    // blockstore: new IDBBlockstore("/ipfs-js"), // create a blockstore - IndexedDB
    blockstore: new MemoryBlockstore(),
    libp2p,
})

console.info("running Helia node...");

// create a filesystem on top of Helia, in this case it's UnixFS
export const heliaFS = mfs(helia)

// turn strings into Uint8Arrays
// const encoder = new TextEncoder()
 // WRITE
    // curl -X POST -F file=@test.txt "http://127.0.0.1:5001/api/v0/files/write?arg=/test.txt&create=true"

    // STAT
    // curl -X POST "http://127.0.0.1:5001/api/v0/files/stat?arg=/test.txt"

    // CHCID
    // curl -X POST "http://127.0.0.1:5001/api/v0/files/chcid?arg=/test.txt"

    // READ
    // curl -X POST "http://127.0.0.1:5001/api/v0/files/read?arg=/test.txt"

    // CP
    // curl -X POST "http://127.0.0.1:5001/api/v0/files/cp?arg=/test.txt&arg=/test2.txt"

    // MKDIR
    // curl -X POST "http://127.0.0.1:5001/api/v0/files/mkdir?arg=/new-dir"

    // MV
    // curl -X POST "http://127.0.0.1:5001/api/v0/files/mv?arg=/test2.txt&arg=/new-dir/test2.txt"

    // RM
    // curl -X POST "http://127.0.0.1:5001/api/v0/files/rm?arg=/new-dir&force=true"

    // LS
    // curl -X POST "http://127.0.0.1:5001/api/v0/files/ls?arg=/"

// fs.addBytes

// FilesWrite
// await heliaFS.writeBytes(encoder.encode('Hello World 201'), "/abc.txt")

// FilesChCid not supported
// heliaFS.chmod("/")

// FilesStat
const stat = await heliaFS.stat("/");

console.log(stat);

// fs.cat
// fs.cp
// fs.mkdir
// // fs.mv
// fs.rm
// fs.ls


