use crate::contact_center::skill::Skill;

// Agent
// - has a set of skills
// - Can answer a call
// - The more calls they answer, the better at handling those calls
// - Have a happiness factor (which influences the customers experience)
// - Can be trained to handle different calls, which makes them more effecient in answering those
//   calls

#[derive(Default, Clone)]
pub struct Agent {
    pub id: usize,
    skills: Vec<Skill>,
}
