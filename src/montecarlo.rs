use rand::{
    prelude::{IteratorRandom, SliceRandom},
    Rng,
};

use crate::board::{Action, Board, Color, StaticList};
use indextree::{Arena, NodeId};

#[derive(Clone, Copy)]
struct NodeState {
    board: NodeId,
    sims: i32,
    wins: i32,
    action_taken: Option<Action>,
    loc: Option<NodeId>,
}

const UCT_CONST: f32 = 1.141;
pub struct Tree {
    root: NodeId,
    arena: Arena<NodeState>,
    board_arena: Arena<Board>,
}

impl Tree {
    pub fn get_monte_carlo_move(&mut self) -> Action {
        let root = self.arena.get(self.root).unwrap().get();
        let starting_moves = self
            .board_arena
            .get(root.board)
            .unwrap()
            .get()
            .get_all_actions();

        if starting_moves.len() == 1 {
            return starting_moves.get(0);
        }
        root.expand(&mut self.arena, &mut self.board_arena);
        for _ in 0..10000 {
            self.expand_tree();
        }
        return self.select_best_move();
    }

    pub fn new(board: Board) -> Self {
        let mut board_arena = Arena::<Board>::new();
        let root = NodeState::new(board, &mut board_arena);
        let mut arena = Arena::new();
        let root_id = arena.new_node(root);
        arena.get_mut(root_id).unwrap().get_mut().set_loc(root_id);
        Self {
            root: root_id,
            arena: arena,
            board_arena: board_arena,
        }
    }

    pub fn select_best_move(&self) -> Action {
        let children = self.root.children(&self.arena);

        let max = children.into_iter().max_by(|node_id, node_id2| {
            self.arena
                .get(*node_id)
                .unwrap()
                .get()
                .sims
                .cmp(&self.arena.get(*node_id2).unwrap().get().sims)
        });
        return self
            .arena
            .get(max.unwrap())
            .unwrap()
            .get()
            .action_taken
            .unwrap();
    }

    pub fn expand_tree(&mut self) {
        let mut arena = &mut self.arena;
        let promising_node_id = arena.get(self.root).unwrap().get().select_node(&arena);

        // println!("selected node {}", promising_node_id);

        let promising_node = arena.get_mut(promising_node_id).unwrap().get_mut();

        promising_node.expand(&mut arena, &mut self.board_arena);

        let children = promising_node_id.children(&self.arena).into_iter();

        let mut list = StaticList::new();

        for child in children {
            list.push(child);
        }

        let children = list;

        let mut test_node = promising_node_id;

        if children.len() > 0 {
            //     println!(
            //         "number of children {} nodeId {}",
            //         children.len(),
            //         promising_node_id
            //     );
            let index = rand::thread_rng().gen_range(0..children.len());
            test_node = children.get(index);
        }

        self.arena
            .get(test_node)
            .unwrap()
            .get()
            .play_out(&mut self.arena, &mut self.board_arena);
    }
}

impl<'a, 'b> NodeState {
    pub fn new(board: Board, arena: &mut Arena<Board>) -> Self {
        Self {
            board: arena.new_node(board),
            sims: 0,
            wins: 0,
            action_taken: None,
            loc: None,
        }
    }

    pub fn new_child(board: NodeId, action: Action, board_arena: &mut Arena<Board>) -> Self {
        let mut board = board_arena.get(board).unwrap().get().clone();
        board.execute_action(action);
        Self {
            board: board_arena.new_node(board),
            sims: 0,
            wins: 0,
            action_taken: Some(action),
            loc: None,
        }
    }

    fn set_loc(&mut self, loc: NodeId) {
        self.loc = Some(loc)
    }

    pub fn expand(self, arena: &'a mut Arena<NodeState>, board_arena: &'b mut Arena<Board>) {
        // println!("expanding on node {:?}", self.loc);
        let moves = board_arena.get(self.board).unwrap().get().get_all_actions();
        for index in 0..moves.len() {
            let action = moves.get(index);
            let new_child = arena.new_node(NodeState::new_child(self.board, action, board_arena));
            arena
                .get_mut(new_child)
                .unwrap()
                .get_mut()
                .set_loc(new_child);
            self.loc.unwrap().append(new_child, arena);
        }
    }

    pub fn uct_value(&self, node_id: NodeId, arena: &Arena<NodeState>) -> f32 {
        //NOTE: might want to implement caching of uct values
        let parent_sims = match arena.get(node_id).unwrap().parent() {
            Some(parent) => arena.get(parent).unwrap().get().sims,
            None => 1,
        };
        if self.sims == 0 {
            return f32::INFINITY;
        }
        self.wins as f32 / self.sims as f32
            + (UCT_CONST * ((parent_sims as f32) / (self.sims as f32)))
                .log2()
                .sqrt()
    }

    pub fn play_out(self, arena: &mut Arena<NodeState>, board_arena: &Arena<Board>) {
        let mut copy_board = board_arena.get(self.board).unwrap().get().clone();
        let mut winner = None;
        while winner == None {
            winner = copy_board.make_random_move();
        }
        self.back_propagate(winner.unwrap(), arena, board_arena);
    }

    pub(crate) fn back_propagate(
        self,
        winning: Color,
        arena: &mut Arena<NodeState>,
        board_arena: &Arena<Board>,
    ) {
        let board = board_arena.get(self.board).unwrap().get();
        let mut self_node = arena.get_mut(self.loc.unwrap()).unwrap().get_mut();

        match board.get_last_turn() {
            Some(last_turn) => {
                if last_turn == winning {
                    self_node.wins += 1;
                }
            }
            None => {}
        }

        self_node.sims += 1;

        match arena.get(self.loc.unwrap()).unwrap().parent() {
            Some(parent) => {
                arena
                    .get(parent)
                    .unwrap()
                    .get()
                    .back_propagate(winning, arena, board_arena)
            }
            None => {}
        }
    }

    pub fn select_node(&self, arena: &Arena<NodeState>) -> NodeId {
        let select = self
            .loc
            .unwrap()
            .children(arena)
            .into_iter()
            .max_by(|node1, node2| {
                arena
                    .get(*node1)
                    .unwrap()
                    .get()
                    .uct_value(*node1, arena)
                    .partial_cmp(&arena.get(*node2).unwrap().get().uct_value(*node2, arena))
                    .unwrap()
            });

        match select {
            Some(node_id) => {
                let new_node = arena.get(node_id).unwrap().get();
                new_node.select_node(arena)
            }
            None => self.loc.unwrap(),
        }
    }
}
