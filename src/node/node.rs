pub mod node {

    use crate::Wallet;
    use crate::Chain;
    use crate::node::gossip::gossip;

    pub struct Role {
        Tracker,
        Node,
        Miner,
    }
    
    pub struct Node {
        id: Uuid,
        role: Role,
        wallet: Wallet,
        chain: Chain,
        neighbours: Vec<Neighbour>,
        client: NetClient,
    }

    fn enterNetwork(&self, id: String, trackers: Vec<String>) -> Result<(), EnterAttemptError> {
        let cleared = false;
        for tracker in trackers:
            match gossip::greet(tracker) {
                Ok(neighbour) => {
                    self.neighbours.push(neighbour);
                    cleared = true,
                },
                Err(e) => continue,
            }
        }
        if !cleared {
            Err(EnterAttemptedError)
        }
        Ok(())
    }

    fn leaveNetwork(&self) {
        for neighbour in self.neighbours {
            gossip::farewell(neighbour);
        }
    }         


    fn submitTransaction(&self, transaction: &Transaction) {
        self.neighbours
            .iter()
            .filter(|neighbour| { neighbour.role == Role::Miner })
            .map(|miner| { gossip::sendTransaction(miner, &transaction) })
            .collect();
    }

    fn updateChain(&self) {
        let cursor = self.neighbours.iter();
        let cur_neighbour = cursor.next();
        while cur_neighbour != None {
            match gossip::poll_chain(cur_neighbour) {
                Ok(chain) => return Ok(chain),
                Err(e) => cur_neighbour = cursor.next(),
            }
        }
        Err(UpdateChainError)
    }

    fn gossip(&self) {
        gossip::wait_gossip_interval();
        let random_neighbours = self.get_random_neighbours();
        for neighbour in random_neighbours {
            gossip::sendChain(neighbour, self.chain);
            gossip::sendNewNeighbours(neighbour, self.new_neighbours);
        }
    }

    fn listen(&self) {}
}

    
