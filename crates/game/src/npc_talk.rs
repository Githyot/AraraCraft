//! What the town folk say (plan/npcs-analise.md, lote N1).
//!
//! Right click on a townsperson and they turn, speak a line in the dialogue box (their mouth
//! moving, in their own voice) and the box says what a second click does: open the shop, get
//! healed. Lines come from three places:
//! - each NPC's own lines, in their own manner;
//! - the moment: night, rain, a blood moon, a party, hard mode, the player hurt;
//! - the neighbours they love or can't stand (tr_town.rs MOODS).
//!
//! All text is ours, written for this game.

use glam::Vec3;
use voxel::mob::Act;

use crate::Game;

/// Seconds a second click still counts as "go on" after the first.
const FOLLOW_UP: f32 = 15.0;

#[derive(Default)]
pub struct NpcTalk {
    /// The NPC the dialogue is with (network id) and when it began.
    with: u32,
    at: f32,
    /// Lines said lately, so the same one doesn't come twice in a row.
    last: Vec<String>,
}

/// Each NPC's own lines.
const LINES: &[(&str, &[&str])] = &[
    ("npc_guide", &["Se tiver dúvida de qualquer coisa, pode perguntar. Eu já li um pouco de tudo.", "Uma bancada, uma tocha e um teto: é assim que começa toda grande aventura."]),
    ("npc_merchant", &[
        "Compro barato, vendo honesto. Bom, quase sempre honesto.",
        "Tochas, baldes, sementes... tudo o que um aventureiro esquece de levar.",
        "Moeda guardada é moeda que não some quando você cai num buraco.",
        "Já viajei por sete vilas antes desta. Nenhuma tinha um cliente tão bom.",
        "Cuidado com o mercador do esqueleto lá nas cavernas. Os preços dele são de matar.",
    ]),
    ("npc_nurse", &[
        "Fique parado. Isso aqui vai arder um pouquinho.",
        "Você cai de cada altura... Eu deveria cobrar por queda, não por ponto de vida.",
        "Lave as mãos depois de mexer em carne podre. Sério.",
        "Uma poção de cura no bolso salva mais vidas do que qualquer espada.",
        "Não, eu não curo picada de abelha de graça. Ninguém mandou mexer na colmeia.",
    ]),
    ("npc_wizard", &[
        "A mana volta sozinha, a paciência nem sempre.",
        "Gemas guardam luz. Varinhas só ensinam a luz a sair por uma ponta.",
        "Já transformei um sapo em príncipe. Ele preferia ser sapo.",
        "Leia os tomos com calma. Pular página dá choque, literalmente.",
        "Sinto cheiro de ozônio... vai chover, ou alguém mexeu nos meus raios.",
    ]),
    ("npc_smith", &[
        "Metal bom canta quando o martelo bate. Ouviu? Esse aí desafina.",
        "Traga minério e paciência. O resto o fogo resolve.",
        "Bronze, prata, tungstênio, platina... cada mundo esconde dois de cada par.",
        "Quer uma lâmina afiada? Não use ela pra cavar terra.",
        "Meu avô forjava ferraduras. Eu forjo o que derruba guardiões.",
    ]),
    ("npc_farmer", &[
        "Terra arada perto da água cresce o dobro. A natureza é generosa com quem cuida.",
        "Abóbora boa é aquela que ninguém pisou.",
        "Farinha de osso faz milagre, mas não abuse.",
        "As vacas gostam de trigo, as galinhas de sementes. Eu gosto de pão quente.",
        "Choveu ontem, a plantação agradece.",
    ]),
    ("npc_guard", &[
        "Fico de olho nos monstros por aqui. Durma tranquilo.",
        "Muro alto e tocha acesa: metade da guarda é isso.",
        "Já enfrentei três zumbis de uma vez. Bom, dois e meio, um estava sem perna.",
        "Se ouvir chocalho de osso à noite, feche a porta.",
    ]),
    ("goblin_trader", &["Coisa boa, preço bom, goblin feliz!", "Não pergunte de onde veio. Goblin acha, goblin vende.", "Moeda brilha bonito. Goblin gosta."]),
    ("npc_tinkerer", &["Acessório bom é acessório misturado. Dois viram um, um vira melhor.", "Reforjar é como jogar dados, só que com martelo.", "Me tiraram da corda e eu fiquei. Gratidão de goblin é coisa rara."]),
    ("npc_mechanic", &["Fio vermelho no lugar certo e a casa se abre sozinha.", "Alavancas, placas de pressão, pistões... dá pra fazer quase tudo com paciência.", "Aquele velho da masmorra me prendeu por anos. Valeu por me soltar."]),
    ("npc_sorceress", &["A magia antiga está nas pedras. É só saber escutar.", "Feitiço errado e você vira estátua. Eu já vi acontecer.", "As estrelas mudaram de lugar esta noite. Algo grande está acordando."]),
    ("npc_dryad", &[
        "A floresta lembra de cada árvore cortada. Plante outra, por favor.",
        "Sinto a terra doente para os lados da corrupção. Pó de purificação ajuda.",
        "As abelhas me contam tudo. Inclusive que você roubou mel.",
        "Toda semente quer ser floresta. Só precisa de tempo.",
    ]),
    ("npc_demolitionist", &[
        "Explosivo bom é o que explode na hora certa. E longe de mim.",
        "Cuidado onde joga essa bomba. Eu ainda quero minhas sobrancelhas.",
        "Mineração com picareta é bonito. Com dinamite é rápido.",
        "Ontem testei um pavio novo. A parede do vizinho discorda dos resultados.",
    ]),
    ("npc_arms_dealer", &[
        "Bala de mosquete não erra quem mira com calma.",
        "Arma limpa, mão firme, alvo longe. Essa é a regra.",
        "Monstro grande? Arma grande. A matemática é simples.",
        "Não, não vendo pólvora fiado.",
    ]),
    ("npc_angler", &[
        "Peixe grande morde de manhã cedo. Ou à tarde. Ou quando quer.",
        "Você está espantando os peixes só de ficar parado aí.",
        "Um dia eu pesco o peixe mais estranho do mundo. Ou você pesca pra mim.",
    ]),
    ("npc_stylist", &["Esse cabelo já viu dias melhores, hein?", "Uma tesoura boa vale mais que uma espada. Pelo menos aqui no salão.", "Cor nova, vida nova!"]),
    ("npc_dye_trader", &["Cada flor esconde uma cor. Eu só ajudo ela a sair.", "Azul do mar, vermelho do fogo, verde da selva... qual vai ser hoje?", "Roupa sem cor é como dia sem sol."]),
    ("npc_painter", &["A luz da tarde aqui é perfeita para pintar.", "Uma parede vazia é um quadro esperando.", "Tinta na roupa é sinal de trabalho bem feito."]),
    ("npc_tailor", &["Um ponto bem dado dura a vida inteira.", "Lã branca aceita qualquer cor. Seja como a lã.", "Aquela maldição me prendeu por anos. Agora eu costuro em paz."]),
    ("npc_witch_doctor", &["Veneno e remédio são a mesma coisa. Muda a dose.", "A selva fala. Quem escuta, vive mais.", "Os esporos da selva cantam à noite. Você não ouve?"]),
    ("npc_zoologist", &["Todo bicho tem seu jeito. Até o mais feio.", "Não acaricie o lobo selvagem. Ele não leu as regras.", "Já catalogou quantas criaturas hoje? Eu vou em trezentas!"]),
    ("npc_party_girl", &["Festa! Agora! Por que não?", "Confete resolve quase tudo.", "Uma vila sem música é só um monte de casas."]),
    ("npc_pirate", &["Arrr, o mar é o único lugar honesto que eu conheço.", "Já perdi um olho, um navio e três papagaios. O tesouro ninguém tira.", "Terra firme me dá enjoo, marujo."]),
    ("npc_steampunker", &["Vapor, engrenagens e um pouco de sorte. É tudo de que se precisa.", "Esse trilho aí pede um vagão novo.", "Se apitar, não toque. Se não apitar, também não."]),
    ("npc_cyborg", &["Sistemas operando. Humor: sessenta e dois por cento.", "Meus circuitos detectam monstros a quarenta blocos. Só por precaução.", "Foguete limpo é foguete que voa reto."]),
    ("npc_princess", &["Que vila linda vocês construíram!", "Uma princesa precisa de amigos, não de castelos.", "Bolo é sempre uma boa ideia."]),
    ("npc_peddler", &["Coisas raras, de muito longe. Só hoje, só até o sol se pôr.", "Não guarde moeda pra amanhã, eu não vou estar aqui amanhã.", "Cada estrada me dá uma coisa estranha para vender."]),
    ("npc_mycologist", &["Cogumelo que brilha não é para comer. Ou é? Ainda estou testando.", "O mundo de baixo é cheio de vida que ninguém vê.", "Esporos no nariz dão uma tosse... azul."]),
    ("npc_winter_elder", &["Já vi mais invernos do que você viu dias.", "Neve fresca cobre todos os rastros. Bons e ruins.", "Um biscoito quente vale mais que ouro numa noite fria."]),
    ("npc_tax_collector", &["Imposto é imposto. Até para quem mora num buraco.", "Hmpf. Pelo menos vocês pagam em dia.", "Fui liberto do submundo para isto? Contar moedas? ...Até que gosto."]),
    ("npc_tavernkeep", &["Senta aí, a cerveja está gelada. Bom, quase gelada.", "Todo herói passa pela taverna. Os sábios voltam.", "Ouvi dizer que tem um exército velho rondando os cristais."]),
    ("npc_golfer", &["Respira, olha a bandeira, bate macio.", "Buraco em uma! Bom, quase."]),
];

/// Lines for the moment, said by anyone (the NPC's own manner comes from the lines above).
const NIGHT: &[&str] = &["Já é tarde. Melhor ficar dentro de casa, monstros andam por aí.", "A noite está cheia de barulhos estranhos.", "Boa noite. Amanhã a gente conversa melhor."];
const RAIN: &[&str] = &["Que chuva! Pelo menos as plantas estão felizes.", "Entre um pouco, você está pingando no meu chão.", "Dia bom para ficar em casa ouvindo a chuva."];
const BLOOD: &[&str] = &["A lua está vermelha... ninguém sai de casa esta noite!", "Ouviu isso? Os mortos estão inquietos com essa lua.", "Tranque a porta. Lua de sangue não é brincadeira."];
const PARTY: &[&str] = &["Festa na vila! Pegue um pedaço de bolo.", "Até eu estou dançando hoje!", "Confete no cabelo, música no ar. Que dia!"];
const HURT: &[&str] = &["Você está ferido! Procure a Enfermeira, ou pelo menos coma alguma coisa.", "Que corte feio... sente aqui um pouco.", "Você está pálido. Cuide dessa vida antes de sair de novo."];
const HARD: &[&str] = &["Desde que aquele muro de carne caiu, o mundo está diferente. Mais perigoso.", "Sinto uma luz estranha e uma escuridão nova espalhando pela terra.", "Os minérios novos brilham nas cavernas. E coisas novas também."];

/// Each settler of the villages: their own lines.
const SETTLER_LINES: &[(&str, &[&str])] = &[
    ("settler_farmer", &["Trigo de sobra? Eu compro.", "Colheita boa este ano. Os corvos que se cuidem.", "Pão fresquinho saindo do forno."]),
    ("settler_smith", &["Carvão mantém a forja viva. Traga que eu pago.", "Ferro bom, preço justo.", "Cuidado com as faíscas!"]),
    ("settler_librarian", &["Papel é memória. Traga papel e eu pago bem.", "Silêncio na biblioteca, por favor. Ah, aqui não tem biblioteca? Mesmo assim.", "Um mapa vale mais que mil palavras."]),
    ("settler_shepherd", &["Lã branca, macia como nuvem.", "As ovelhas sabem quando vai chover.", "Tosquio com cuidado. Ovelha feliz dá lã melhor."]),
];

/// The NPC's own lines (an empty list when it has none).
pub(crate) fn own_lines(name: &str) -> &'static [&'static str] {
    LINES.iter().chain(SETTLER_LINES).find(|l| l.0 == name).map_or(&[], |l| l.1)
}

impl Game {
    /// Every line this NPC could say now, the moment's lines first.
    fn npc_lines(&self, name: &str) -> (Vec<String>, Vec<String>) {
        let mut moment: Vec<&str> = Vec::new();
        if self.t5.hard.blood_moon() {
            moment.extend(BLOOD);
        } else if self.t5.tr.town.party > 0.0 {
            moment.extend(PARTY);
        } else if self.weather.state.raining {
            moment.extend(RAIN);
        } else if self.is_night() {
            moment.extend(NIGHT);
        }
        if self.survival && self.health < self.max_health * 0.35 {
            moment.extend(HURT);
        }
        let mut own: Vec<String> = own_lines(name).iter().map(|s| s.to_string()).collect();
        if self.t5.hard.on {
            own.extend(HARD.iter().map(|s| s.to_string()));
        }
        // Neighbours they love or can't stand, when they live close.
        if let (Some(&(_, _, loves, hates)), Some(me)) = (crate::tr_town::MOODS.iter().find(|m| m.0 == name), self.town.npcs.iter().find(|n| n.name == name)) {
            let home = Vec3::from(me.home);
            let dn = |n: &str| self.mobs.reg.id(n).map(|i| self.mobs.reg.defs[i].display_name.clone()).unwrap_or_default();
            for n in self.town.npcs.iter().filter(|n| Vec3::from(n.home).distance(home) < 25.0) {
                if loves.contains(&n.name.as_str()) {
                    own.push(format!("Ter {} como vizinho deixa qualquer dia melhor.", dn(&n.name)));
                }
                if hates.contains(&n.name.as_str()) {
                    own.push(format!("{} fez barulho a noite inteira de novo. Não aguento mais.", dn(&n.name)));
                }
            }
        }
        (moment.into_iter().map(String::from).collect(), own)
    }

    /// Pick a line for this NPC, not one said lately.
    pub(crate) fn npc_line(&mut self, name: &str) -> Option<String> {
        let (moment, own) = self.npc_lines(name);
        let r = self.surv.rng.f32();
        let pool = if !moment.is_empty() && (own.is_empty() || r < 0.55) { moment } else { own };
        if pool.is_empty() {
            return None;
        }
        let start = (self.surv.rng.f32() * pool.len() as f32) as usize % pool.len();
        let k = (0..pool.len()).map(|d| (start + d) % pool.len()).find(|&k| !self.npc_talk.last.contains(&pool[k])).unwrap_or(start);
        let line = pool[k].clone();
        self.npc_talk.last.push(line.clone());
        if self.npc_talk.last.len() > 8 {
            self.npc_talk.last.remove(0);
        }
        Some(line)
    }

    /// The NPC at `i` speaks: dialogue box, moving mouth, its voice. `hint` is what a second
    /// click does, shown under the line.
    pub(crate) fn npc_speak(&mut self, i: usize, text: String, hint: Option<&str>) {
        let m = &self.mobs.list[i];
        let def = &self.mobs.reg.defs[m.def];
        let (name, dn, at, id) = (def.name.clone(), def.display_name.clone(), m.body.pos, m.net_id);
        let words = text.split_whitespace().count() as f32;
        let full = match hint {
            Some(h) => format!("{text} [{h}]"),
            None => text,
        };
        self.say(&dn, full, at);
        let m = &mut self.mobs.list[i];
        m.voice_t = (words * 0.28).clamp(1.2, 4.5);
        m.sleeping = false;
        m.lying = false;
        if m.social.act != Act::None {
            m.social.act = Act::None;
        }
        // Stand and face the player while talking.
        m.go_to(m.body.pos, 3.0);
        self.npc_talk.with = id;
        self.npc_talk.at = self.now;
        if let Some(fx) = crate::mob_sounds::pick(&name, crate::sfx_extra::Call::Idle, 1.0, self.surv.rng.f32(), self.surv.rng.f32()) {
            self.sfx_at(crate::audio::Sfx::Fx(fx), at + Vec3::Y * 1.5, 0.7, 16.0);
        }
    }

    /// The click is the second one on the same NPC (the dialogue is still open): go on to
    /// the shop, the healing...
    pub(crate) fn npc_follow_up(&self, i: usize) -> bool {
        self.npc_talk.with == self.mobs.list[i].net_id && self.now - self.npc_talk.at < FOLLOW_UP && self.npc_talk.with != 0
    }

    pub(crate) fn npc_talk_done(&mut self) {
        self.npc_talk.with = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_town_seller_has_lines() {
        for (name, _) in crate::npcs::SHOPS.iter().chain(crate::tr_town::SHOPS) {
            assert!(!own_lines(name).is_empty(), "{name} has nothing to say");
        }
        for (name, _, _) in crate::villages::SETTLERS.iter() {
            assert!(!own_lines(name).is_empty(), "{name} has nothing to say");
        }
    }

    #[test]
    fn lines_are_short_enough_for_the_box() {
        for (_, lines) in LINES.iter().chain(SETTLER_LINES) {
            for l in lines.iter() {
                assert!(l.chars().count() < 140, "too long: {l}");
            }
        }
    }
}
