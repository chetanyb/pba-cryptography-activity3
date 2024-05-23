use rand::rngs::OsRng;
use schnorrkel::{signing_context, Keypair, PublicKey, Signature};

struct Player {
    keypair: Keypair,
    pub vrf_output: Option<[u8; 32]>,
    selected_input: Option<u32>,
    pub selected_input_signature: Option<Signature>,
}

impl Player {
    fn new() -> Self {
        let keypair = Keypair::generate_with(OsRng);
        Player {
            keypair,
            vrf_output: None,
            selected_input: None,
            selected_input_signature: None,
        }
    }

    pub fn public_key(&self) -> PublicKey {
        self.keypair.public
    }

    fn select_input(&mut self) {
        let ctx = signing_context(b"Selecting input");
        let random: u32 = rand::random();
        let input_signature = self.keypair.sign(ctx.bytes(&random.to_le_bytes()));

        self.selected_input_signature = Some(input_signature);
        self.selected_input = Some(random);
    }

    fn reveal_input(&self) -> u32 {
        self.selected_input.expect("Input not selected")
    }

    fn draw_card(&mut self, input: &[u8]) {
        let ctx = signing_context(b"drawing card");
        let (vrf_output, _, __) = self.keypair.vrf_sign(ctx.bytes(input));
        self.vrf_output = Some(vrf_output.output.to_bytes());
    }

    fn reveal_card(&self) -> Option<u8> {
        self.vrf_output.map(|output| {
            // Use the VRF output modulo 52 to get a card number
            let card = u8::from_le_bytes([output[0]]) % 52;
            card
        })
    }
}

fn main() {
    // Create players
    let mut players: Vec<Player> = vec![Player::new(), Player::new(), Player::new()];

    // Select inputs
    for player in &mut players {
        player.select_input();
    }

    // Verify selected inputs from users
    for player in &players {
        let ctx = signing_context(b"Selecting input");
        let input = player.selected_input.expect("Input not selected");
        let input_signature = player.selected_input_signature.expect("Signature not found");
        player
            .public_key()
            .verify(ctx.bytes(&input.to_le_bytes()), &input_signature)
            .expect("Invalid input signature");
    }

    let combined_input: u32 = players.iter().map(|player| player.reveal_input()).sum();

    // Players draw cards
    for player in &mut players {
        player.draw_card(&combined_input.to_le_bytes());
    }

    // Reveal cards and determine the winner
    let mut max_card = 0;
    let mut winner_index = 0;

    for (index, player) in players.iter().enumerate() {
        if let Some(card) = player.reveal_card() {
            println!("Player {} drew card: {}", index + 1, card);
            if card > max_card {
                max_card = card;
                winner_index = index;
            }
        }
    }

    println!("Player {} wins with card: {}", winner_index + 1, max_card);
}
