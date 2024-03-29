use std::collections::HashMap;
use std::fmt::Debug;
use std::vec::Vec;
use robotics_lib::world::tile::{Tile, TileType, Content};
use robotics_lib::interface::Direction;
use std::collections::{BinaryHeap};
use std::cmp::Ordering;
use robotics_lib::interface::discover_tiles;
use robotics_lib::interface::Tools;
use robotics_lib::world::World;
use robotics_lib::runner::Runnable;

pub struct BestPath{

}

impl Tools for BestPath{

}

impl BestPath{
    /// Computes the shortest path for a robot through a given world.
    ///
    /// This function calculates the shortest path that the provided robot can take
    /// within the specified world, given a set of known nodes and nodes of interest.
    ///
    /// # Arguments
    ///
    /// * `robot` - A mutable reference to a robot that implements the `Runnable` trait.
    ///             The robot is used to navigate through the world.
    /// * `world` - A mutable reference to the world where the robot will navigate.
    /// * `nodi_conosciuti` - A vector containing tuples of known nodes, each tuple
    ///                       consisting of a position `(i32, i32)` and a `Tile`.
    /// * `nodi_interesse` - A vector containing positions `(i32, i32)` of nodes that
    ///                      the robot is interested in reaching.
    /// * `starting_node` - The starting position `(i32, i32)` for the robot's path.
    /// * `discover` - A boolean flag indicating whether the robot should discover
    ///                new nodes during its pathfinding.
    ///
    /// # Returns
    ///
    /// A vector of vectors, where each inner vector represents a sequence of
    /// `Direction` that the robot should follow to navigate from the starting node
    /// to a node of interest. The outer vector contains all such sequences.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use robotics_lib::energy::Energy;
    /// use robotics_lib::event::events::Event;
    /// use robotics_lib::interface::Tools;
    /// use robotics_lib::runner::backpack::BackPack;
    /// use robotics_lib::runner::{Robot, Runnable, Runner};
    /// use robotics_lib::world::coordinates::Coordinate;
    /// use robotics_lib::world::environmental_conditions::EnvironmentalConditions;
    /// use robotics_lib::world::environmental_conditions::WeatherType::{Rainy, Sunny};
    /// use robotics_lib::world::tile::TileType::Grass;
    /// use robotics_lib::world::tile::{Content, Tile};
    /// use robotics_lib::world::world_generator::Generator;
    /// use robotics_lib::world::World;
    /// use holy_crab_best_path::BestPath;
    ///
    /// fn main() {
    ///     struct MyRobot(Robot);
    ///     struct WorldGenerator {
    ///         size: usize,
    ///     }
    ///     impl WorldGenerator {
    ///         fn init(size: usize) -> Self {
    ///             WorldGenerator { size }
    ///         }
    ///     }
    ///     impl Generator for WorldGenerator {
    ///         fn gen(
    ///             &mut self,
    ///         ) -> (
    ///             Vec<Vec<Tile>>,
    ///             (usize, usize),
    ///             EnvironmentalConditions,
    ///             f32,
    ///             Option<HashMap<Content, f32>>,
    ///         ) {
    ///             let mut map: Vec<Vec<Tile>> = Vec::new();
    ///             // Initialize the map with default tiles
    ///             for _ in 0..self.size {
    ///                 let mut row: Vec<Tile> = Vec::new();
    ///                 for _ in 0..self.size {
    ///                     let tile_type = Grass;
    ///                     let content = Content::None;
    ///                     row.push(Tile {
    ///                         tile_type,
    ///                         content,
    ///                         elevation: 0,
    ///                     });
    ///                 }
    ///                 map.push(row);
    ///             }
    ///             let environmental_conditions = EnvironmentalConditions::new(&[Sunny, Rainy], 15, 12).unwrap();
    ///
    ///             let max_score = rand::random::<f32>();
    ///
    ///             (map, (0, 0), environmental_conditions, max_score, None)
    ///         }
    ///     }
    ///     impl Runnable for MyRobot {
    ///         fn process_tick(&mut self, world: &mut World) {
    ///             /// Real test
    ///             let nodi_conosciuti: Vec<((i32, i32), Tile)> = vec![];
    ///             let nodi_interesse: Vec<(i32, i32)> = vec![(1,3), (3,3), (3,1)];
    ///             println!("{:?}", BestPath::shortest_path(self, world, &nodi_conosciuti, nodi_interesse, (0,0), true ));
    ///             // prints: [[Right, Right, Right, Down], [Down, Down], [Left, Left]]
    ///         }
    ///
    ///         fn handle_event(&mut self, event: Event) {
    ///             println!();
    ///             println!("{:?}", event);
    ///             println!();
    ///         }
    ///
    ///         fn get_energy(&self) -> &Energy {
    ///             &self.0.energy
    ///         }
    ///         fn get_energy_mut(&mut self) -> &mut Energy {
    ///             &mut self.0.energy
    ///         }
    ///
    ///         fn get_coordinate(&self) -> &Coordinate {
    ///             &self.0.coordinate
    ///         }
    ///         fn get_coordinate_mut(&mut self) -> &mut Coordinate {
    ///             &mut self.0.coordinate
    ///         }
    ///
    ///         fn get_backpack(&self) -> &BackPack {
    ///             &self.0.backpack
    ///         }
    ///         fn get_backpack_mut(&mut self) -> &mut BackPack {
    ///             &mut self.0.backpack
    ///         }
    ///     }
    ///
    ///     let r = MyRobot(Robot::new());
    ///     struct Tool;
    ///     impl Tools for Tool {}
    ///     let mut generator = WorldGenerator::init(100);
    ///     let run = Runner::new(Box::new(r), &mut generator);
    ///
    ///     //Known bug: 'check_world' inside 'Runner::new()' fails every time
    ///     match run {
    ///         | Ok(mut r) => {
    ///             let _ = r.game_tick();
    ///         }
    ///         | Err(e) => println!("{:?}", e),
    ///     }
    /// }
    /// ```
    ///
    pub fn shortest_path(robot: &mut impl Runnable, world: &mut World, nodi_conosciuti: &Vec<((i32, i32), Tile)>, nodi_interesse: Vec<(i32, i32)>, starting_node: (i32, i32),discover: bool)->Vec<Vec<Direction>> {
        // swap x and y due to error
        let mut nodi_interesse_swapped: Vec<(i32, i32)> = vec![];
        for nodo in nodi_interesse{
            nodi_interesse_swapped.push((nodo.1, nodo.0));
        }
        let mut corrected_starting_node = (starting_node.0, starting_node.1);

        let (matrix, minx, miny) = from_vec_to_matrix(robot, world, nodi_conosciuti, &nodi_interesse_swapped, corrected_starting_node, discover);
        // rimuovo l'offset dai nodi d'interesse
        let mut nodi_interesse_corrected: Vec<(i32, i32)> = vec![];
        for nodo in nodi_interesse_swapped{
            nodi_interesse_corrected.push((nodo.0 - minx, nodo.1 - miny));
        }

        let coordinates = get_coordinates(&matrix);
        corrected_starting_node = (corrected_starting_node.1 - minx, corrected_starting_node.0-miny);
        let (matrix_node, mut target_nodes, starting_node) = change_matrix(matrix.clone(), nodi_interesse_corrected, corrected_starting_node);

        target_nodes = find_connected_targets(&matrix_node, starting_node, &target_nodes);

        let results = match build_path(&matrix_node, starting_node, target_nodes, &coordinates) {
            Ok(paths) => paths,
            Err(err) => {
                let to_ret: Vec<Vec<Direction>> = vec![vec![]];
                return to_ret;
            }
        };

        return results;
    }
}

const INF: i32 = std::i32::MAX;

// struct per salvare i nodi nel grafo delle adiacenze
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Node {
    index: usize,
    distance: usize,
}

// implemento new per il nodo
impl Node {
    fn new(index: usize, weight: usize) -> Node {
        Node { index: index, distance: weight }
    }
}

#[derive(Debug)]
struct PathResult {
    pub path: Option<Vec<usize>>,
    pub target_node: usize,
    pub total_cost: i32,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.distance.cmp(&self.distance)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// simulazione funzione discover tiles del wg
/*fn discover_tiles(
    to_discover: &[(usize, usize)],
) -> Result<HashMap<(usize, usize), Option<Tile>>, LibError>{

    let mut hashmap: HashMap<(usize, usize), Option<Tile>> = HashMap::new();

    for cell in to_discover{
        let (x, y) = cell;
        hashmap.insert((*x, *y), Some(Tile{tile_type: TileType::Grass, content: Content::Bush(0), elevation: 2}));
    }

    return Ok(hashmap);

}
*/
// funzione per convertire il vettore dei tiles in una matrice + parte riempitiva
fn from_vec_to_matrix(robot: &mut impl Runnable, world: &mut World, nodi_conosciuti: &Vec<((i32, i32), Tile)>, nodi_interesse: &Vec<(i32, i32)>, starting_node: (i32, i32), discover: bool) -> (Vec<Vec<Tile>>, i32, i32) {
    // se discover == true, riempio la matrice con le discover

    if nodi_interesse.len() == 0{
        panic!("Requested at least one target node");
    }

    let mut nodi_interesse_clone: Vec<(i32, i32)> = nodi_interesse.clone();
    nodi_interesse_clone.push((starting_node.1, starting_node.0));

    let mut i_min_x = i32::MAX;
    let mut i_max_x = 0;
    let mut i_min_y = i32::MAX;
    let mut i_max_y = 0;


    for nodo in nodi_interesse_clone{
        let (x, y) = nodo;
        if x < i_min_x { i_min_x = x; }
        if y < i_min_y { i_min_y = y; }
        if x > i_max_x { i_max_x = x; }
        if y > i_max_y { i_max_y = y; }
    }

    if nodi_conosciuti.len()!=0{
        let Some(((mut min_x, mut min_y), _)) = nodi_conosciuti.get(0) else { panic!("Errore impossibile") };
        let Some(((mut max_x, mut max_y), _)) = nodi_conosciuti.get(0) else { panic!("Errore impossibile") };
        for nodo in nodi_conosciuti{
            let ((x, y), tile) = nodo;
            if *x < min_x { min_x = *x; }
            if *y < min_y { min_y = *y; }
            if *x > max_x { max_x = *x; }
            if *y > max_y { max_y = *y; }
        }

        if min_x < i_min_x{ i_min_x = min_x }
        if min_y < i_min_y{ i_min_y = min_y }
        if max_x > i_max_x{ i_max_x = max_x }
        if max_y > i_max_y{ i_max_y = max_y }
    }

    let mut matrix: Vec<Vec<Tile>> = vec![];

    // riempio la matrice di lava
    let mut mask_matrix: Vec<Vec<(Tile, bool)>> = vec![];
        for _ in i_min_x..=i_max_x {
            let mut row: Vec<Tile> = vec![];
            let mut row_mask: Vec<(Tile, bool)> = vec![];
            for _ in i_min_y..=i_max_y{
                row.push(Tile{tile_type: TileType::Lava, content: Content::None, elevation:0});
                row_mask.push((Tile{tile_type: TileType::Lava, content: Content::None, elevation:0}, false));
            }
            matrix.push(row);
            mask_matrix.push(row_mask);
        }


    // metto nella matrice i tile corretti
    // creo una matrice che i dice quali tile sono veri e quali stimati

    for nodo in nodi_conosciuti{
        let ((x, y), tile) = nodo;
        matrix[(y-i_min_y) as usize][(x-i_min_x) as usize] = tile.clone();
        mask_matrix[(y-i_min_y) as usize][(x-i_min_x) as usize].0 = tile.clone();
        mask_matrix[(y-i_min_y) as usize][(x-i_min_x) as usize].1 = true;
    }


    if !discover{
        return (matrix, i_min_x, i_min_y);
    }

    // parte vecchia
    let mut n_discover = 0;
    let matrix_len = mask_matrix.len();
    let mut new_matrix: Vec<Vec<(Tile, bool)>> = Vec::with_capacity(matrix_len);

    for i in 0..matrix_len {
        let mask_matrix_copy = mask_matrix.clone();
        let row = mask_matrix_copy.get(i).unwrap();
        let row_len = row.len();
        let mut new_row: Vec<(Tile, bool)> = Vec::with_capacity(row_len);

        for j in 0..row_len {
            let (val, known) = row.get(j).unwrap();
            if !known {
                // scopro le celle attorno. Se almeno una è walkable prendo il suo valore e lo salvo (il più grande)
                let neighbor = show_neighbor(&mask_matrix, i as i32, j as i32);
                let max_val = find_max_in_tuple(neighbor);
                // se max val è None, faccio una disover e salvo nella new_matrix
                if max_val.is_none() {
                    n_discover+=1;
                    let mut coordinates = [(i,j)];
                    let new_val = (discover_tiles(robot, world, &mut coordinates).unwrap().get(&(i,j)).unwrap().clone().unwrap(), true);
                    new_row.push(new_val.clone());
                    mask_matrix[i][j] = new_val.clone();
                    // TODO: savare i tile scoperti e ritornarli all'utente
                } else{
                    new_row.push((max_val.unwrap(), false));
                }
            } else {
                new_row.push((val.clone(), true));
            }
        }
        new_matrix.push(new_row);
    }

    // ricostruisco la matrice rimuovendo la dupla
    let mut to_ret: Vec<Vec<Tile>> = vec![];
    for i in i_min_x..=i_max_x{
        let mut row: Vec<Tile> = vec![];
        for j in i_min_y..=i_max_y {
            row.push(new_matrix[(i - i_min_x) as usize][(j - i_min_y) as usize].0.clone());
        }
        to_ret.push(row);
    }

    return (to_ret, i_min_x, i_min_y);
}

// funzione per ritornare il costo più alto delle tiles adiacenti
fn find_max_in_tuple(tuple: (Option<Tile>, Option<Tile>, Option<Tile>, Option<Tile>, Option<Tile>, Option<Tile>, Option<Tile>, Option<Tile>)) -> Option<Tile> {
    // Controlla se tutti gli elementi sono None
    if tuple.0.is_none() &&
        tuple.1.is_none() &&
        tuple.2.is_none() &&
        tuple.3.is_none() &&
        tuple.4.is_none() &&
        tuple.5.is_none() &&
        tuple.6.is_none() &&
        tuple.7.is_none(){
        return None;
    }

    // Inizializza il massimo con il primo valore Some o None se tutti sono None

    let mut max_cost_tile: Option<Tile> = None;
    let mut max_cost = 0;

    for tile_option in [tuple.0, tuple.1, tuple.2, tuple.3, tuple.4, tuple.5, tuple.6, tuple.7].iter() {
        if let Some(tile) = tile_option {
            if tile.tile_type.properties().cost() > max_cost {
                max_cost = tile.tile_type.properties().cost();
                max_cost_tile = Some(tile.clone());
            }
        }
    }

    max_cost_tile
}


// recupero dalla matrice i nodi adiacenti
fn show_neighbor(matrix: &Vec<Vec<(Tile, bool)>>, x: i32, y: i32)->(Option<Tile>, Option<Tile>, Option<Tile>, Option<Tile>, Option<Tile>, Option<Tile>,  Option<Tile>,  Option<Tile>){

    let a = scan_matrix(matrix,x-1, y-1);
    let b = scan_matrix(matrix,x-1, y);
    let c = scan_matrix(matrix,x-1, y+1);
    let d = scan_matrix(matrix,x, y-1);
    let e = scan_matrix(matrix,x, y+1);
    let f = scan_matrix(matrix,x+1, y-1);
    let g = scan_matrix(matrix,x+1, y);
    let h = scan_matrix(matrix,x+1, y+1);

    return (a,b,c,d,e,f,g,h);

}

// recupero dalla matrice i tiles specificati dalla coordinata
fn scan_matrix(matrix: &Vec<Vec<(Tile, bool)>>, x: i32, y: i32)->Option<Tile>{
    if let Some(row) = matrix.get(x as usize){
        let tile = row.get(y as usize);
        if tile.is_none(){
            return None
        }
        if !tile.unwrap().1{
            return None
        }
        return Some(row[y as usize].clone().0);
    } else {
        return None;
    }
}


// passo da matrice a grafo
fn change_matrix(matrix_tile: Vec<Vec<Tile>>, nodi_dinteresse: Vec<(i32, i32)>, startin_node: (i32, i32)) -> (Vec<Vec<Node>>, Vec<usize>, usize) {
    let rows = matrix_tile.len();
    let cols = matrix_tile[0].len();
    let mut matrix_node = vec![vec![]; rows * cols];
    let mut label_node = 0;

    let mut target_nodes = vec![];

    let mut starting_node_index = 0;

    for (x,rows) in matrix_tile.iter().enumerate() {
        for (y,tile) in rows.iter().enumerate() {
            if x as i32 == startin_node.0 && y as i32 == startin_node.1{
                starting_node_index = label_node;
            }
            let is_walkable = is_wakable(tile);
            // se nodo d'interesse pusho
            for node in &nodi_dinteresse {
                let (x2, y2) = node;
                if x == *x2 as usize && y == *y2 as usize {
                    target_nodes.push(label_node);
                }
            }

            if is_walkable {
                let neighbours = get_neighbours(&matrix_tile,x,y,label_node, &tile);
                for i in neighbours {
                    matrix_node[label_node].push(i);
                }
            }
            label_node += 1;
        }
    }
    (matrix_node, target_nodes, starting_node_index)
}

fn get_cost (tile: &Tile) -> usize {
    match &tile.tile_type {
        TileType::DeepWater | TileType::Lava | TileType::Wall => 100000,
        _ => tile.tile_type.properties().cost()
    }
}
/// Returns a boolean corresponding to the attribute walk of Tile
fn is_wakable (tile: &Tile) -> bool {
    match &tile.tile_type {
        TileType::DeepWater | TileType::Lava | TileType::Wall => {return false;}
        _ => true
    }
}

/// Returns the cost of moving to a Tile with higher elevation
fn get_cost_elevation (tile_arrive: &Tile, tile_start: &Tile) -> usize {
    if tile_arrive.elevation <= tile_start.elevation {
        return 0;
    }
    (tile_arrive.elevation - tile_start.elevation).pow(2)
}


/// Returns a vector of Node made by the neighbours of the tile given as a parameter in the function if they are walkable
fn get_neighbours (matrix_tile: &Vec<Vec<Tile>>, x: usize, y: usize, value: usize, tile: &Tile) -> Vec<Node> {
    let rows = matrix_tile.len();
    let cols = matrix_tile[0].len();
    let mut vec = vec![];
    // Tile at bottom
    if (x as i32-1) >= 0 && (x as i32-1) < rows as i32 && (y) < cols {
        if is_wakable(&matrix_tile[x-1][y]) {
            if matrix_tile[x-1][y].elevation == 0 {
                vec.push(Node::new(value-cols,get_cost(&matrix_tile[x-1][y])));
            }
            else {
                vec.push(Node::new(value-cols,get_cost(&matrix_tile[x-1][y]) + get_cost_elevation(&matrix_tile[x-1][y],tile)));
            }
        }
    }
    // Tile at right
    if (x) < rows && (y+1) < cols {
        if is_wakable(&matrix_tile[x][y+1]) {
            if matrix_tile[x][y+1].elevation == 0 {
                vec.push(Node::new(value+1,get_cost(&matrix_tile[x][y+1])));
            }
            else {
                vec.push(Node::new(value+1,get_cost(&matrix_tile[x][y+1]) + get_cost_elevation(&matrix_tile[x][y+1],tile)));
            }
        }
    }
    // Tile at top
    if (x+1) < rows && (y) < cols {
        if is_wakable(&matrix_tile[x+1][y]) {
            if matrix_tile[x+1][y].elevation == 0 {
                vec.push(Node::new(value+cols,get_cost(&matrix_tile[x+1][y])));
            }
            else {
                vec.push(Node::new(value+cols,get_cost(&matrix_tile[x+1][y]) + get_cost_elevation(&matrix_tile[x+1][y],tile)));
            }
        }
    }
    // Tile at left
    if (x) < rows && (y as i32-1) >= 0 && (y as i32-1) < cols as i32 {
        if is_wakable(&matrix_tile[x][y-1]) {
            if matrix_tile[x][y-1].elevation == 0 {
                vec.push(Node::new(value-1,get_cost(&matrix_tile[x][y-1])));
            }
            else {
                vec.push(Node::new(value-1,get_cost(&matrix_tile[x][y-1]) + get_cost_elevation(&matrix_tile[x][y-1],tile)));
            }
        }
    }
    vec
}



// fn print_matrix<T: Debug>(matrix: &Vec<Vec<T>>) {
//     let mut table = Table::new();
//     // Aggiungi una riga vuota per separare le intestazioni dalla tabella
//     table.add_row(Row::empty());
//
//     for row in matrix {
//         let mut table_row = Row::empty();
//
//         for cell in row {
//             let cell_content = format!("{:?}", cell);
//             let cell = Cell::new(&cell_content).style_spec("c");
//             table_row.add_cell(cell);
//         }
//
//         table.add_row(table_row);
//     }
//
//     table.printstd();
// }




/// Performs Dijkstra's algorithm to find the shortest paths from the start node to all other nodes in the graph.
///
/// # Arguments
///
/// * `graph` - The graph represented as a vector of vectors of Node.
/// * `start` - The index of the starting node.
///
/// # Returns
///
/// A tuple containing a vector of the shortest distances from the start node to all the nodes and a vector of Option<usize> representing the predecessors of each node in the shortest path.
fn dijkstra(graph: &Vec<Vec<Node>>, start: usize) -> (Vec<Option<i32>>, Vec<Option<usize>>) {
    let mut distance: Vec<Option<i32>> = vec![None; graph.len()];
    let mut predecessor: Vec<Option<usize>> = vec![None; graph.len()];
    let mut visited: Vec<bool> = vec![false; graph.len()];

    distance[start] = Some(0);
    let mut heap = BinaryHeap::new();
    heap.push(Node { index: start, distance: 0 });

    while let Some(Node { index, distance: dist }) = heap.pop() {
        if visited[index] {
            continue;
        }
        visited[index] = true;

        for neighbor in &graph[index] {
            let new_distance = dist + neighbor.distance;
            let neighbor_distance: usize = distance[neighbor.index].unwrap_or(INF) as usize;

            if new_distance < neighbor_distance {
                distance[neighbor.index] = Some(new_distance as i32);
                predecessor[neighbor.index] = Some(index);
                heap.push(Node { index: neighbor.index, distance: new_distance });
            }
        }
    }

    (distance, predecessor)
}

/// Reconstructs the shortest path from the start node to the target node using the predecessors vector.
fn reconstruct_shortest_path(predecessor: Vec<Option<usize>>, target: usize) -> Option<Vec<usize>> {
    let mut path = Vec::new();
    let mut current = target;

    while let Some(prev) = predecessor[current] {
        path.push(current);
        current = prev;
    }

    path.push(current);
    path.reverse();

    if path.len() == 0 {
        return None;
    }


    if path == [target] {  // If the path only contains the target node, there is no valid path
        None
    } else {
        Some(path)
    }
}

/// Finds the shortest paths from a start node to multiple target nodes in a graph represented as a matrix.
///
/// This function uses Dijkstra's algorithm to calculate the shortest distances and then it reconstructs the paths through the function reconstruct_shortest_path
///
/// # Arguments
///
/// * `graph` - The graph represented as a vector of vectors of Node.
/// * `start` - The index of the starting node.
/// * `target_nodes` - A vector of indices representing the target nodes.
///
/// # Returns
///
/// A vector of `PathResult` structs, each one representing a target node, its respective shortest path, and total cost.
fn find_shortest_paths(graph: &Vec<Vec<Node>>, start: usize, target_nodes: &Vec<usize>) -> Vec<PathResult> {
    let (shortest_distances, predecessors) = dijkstra(graph, start);
    let mut results = Vec::new();

    for target_node in target_nodes {
        let path = reconstruct_shortest_path(predecessors.clone(), *target_node);

        let tmp =match &shortest_distances[*target_node] {
            None => {0}
            Some(t) => {*t}
        };

        let result = PathResult {
            path,
            target_node: *target_node,
            total_cost: tmp,
        };

        results.push(result);
    }

    results
}

/////////////////////////////////////////////////////

/// Costruisce percorsi tra nodi in un grafo utilizzando gli algoritmi più brevi da un punto di partenza a un insieme di nodi destinazione.
///
/// # Parametri
///
/// - `graph`: Il grafo rappresentato come un vettore di vettori di nodi.
/// - `start`: L'indice del nodo di partenza.
/// - `target_nodes`: Un vettore di indici dei nodi destinazione.
/// - `coordinates`: Una mappa che associa indici di nodi alle loro coordinate (riga, colonna).
///
/// # Restituisce
///
/// Un risultato contenente un vettore di vettori di direzioni rappresentanti i percorsi più brevi dai nodi di partenza ai nodi destinazione.
///
/// # Errori
///
/// Restituisce un errore se si verifica un problema durante il calcolo dei percorsi o la conversione delle coordinate in direzioni.
///
fn build_path(graph: &Vec<Vec<Node>>, mut start: usize, mut target_nodes: Vec<usize>, coordinates:&HashMap<usize, (usize,usize)>)
              -> Result<Vec<Vec<Direction>>, &'static str> {
    let mut final_path: Vec<Vec<Direction>> = Vec::new();

    while !target_nodes.is_empty() {
        let paths = find_shortest_paths(graph, start, &target_nodes);

        if let Some(best) = paths.iter().min_by_key(|path| path.total_cost) {
            if let Some(path) = &best.path {
                start = path.last().cloned().unwrap();
                let directions = path_to_directions(coordinates, path)?;
                final_path.push(directions);
                target_nodes.retain(|&x| x != best.target_node);
            }
        }
    }

    Ok(final_path)
}

/// Converte una sequenza di nodi in una sequenza di direzioni basate sulle coordinate fornite.
///
/// # Parametri
///
/// - `coordinates`: Una mappa che associa indici di nodi alle loro coordinate (riga, colonna).
/// - `path`: Un vettore di indici di nodi rappresentanti un percorso nel grafo.
///
/// # Restituisce
///
/// Un risultato contenente un vettore di direzioni corrispondenti al percorso fornito o un errore.
///
/// # Errori
///
/// Restituisce un errore se uno degli indici di nodo nel percorso non è presente nelle coordinate.
///
fn path_to_directions(coordinates: &HashMap<usize, (usize, usize)>, path: &Vec<usize>, ) -> Result<Vec<Direction>, &'static str> {
    let mut directions = Vec::new();

    // Assicurati che il percorso abbia almeno un nodo
    if path.is_empty() {
        return Ok(directions);
    }

    // Itera attraverso il percorso
    for i in 1..path.len() {
        let current_node = path[i - 1];
        let next_node = path[i];

        // Ottieni le coordinate correnti e successive
        let current_coords = coordinates.get(&current_node).ok_or("Coordinate mancanti per il nodo corrente.")?;
        let next_coords = coordinates.get(&next_node).ok_or("Coordinate mancanti per il prossimo nodo.")?;

        // Stampa le coordinate per scopi di debug
        //println!("{:?} {:?}", current_coords, next_coords);

        // Determina la direzione in base al cambiamento di coordinate
        let direction = match (next_coords.0 as i32 - current_coords.0 as i32, next_coords.1 as i32 - current_coords.1 as i32) {
            (-1, 0) => Direction::Up,
            (1, 0) => Direction::Down,
            (0, -1) => Direction::Left,
            (0, 1) => Direction::Right,
            _ => return Err("Cambiamento di coordinate non valido per determinare la direzione."),
        };

        // Aggiungi la direzione al vettore di direzioni
        directions.push(direction);
    }

    Ok(directions)
}

/// Trova i nodi connessi a partire da un nodo di partenza in un grafo e restituisce quelli che sono anche nei nodi di destinazione.
///
/// # Parametri
///
/// - `graph`: Il grafo rappresentato come un vettore di vettori di nodi.
/// - `start`: L'indice del nodo di partenza.
/// - `targets`: Un vettore di indici dei nodi destinazione.
///
/// # Restituisce
///
/// Un vettore contenente gli indici dei nodi connessi che sono anche nei nodi di destinazione.
///
fn find_connected_targets(graph: &Vec<Vec<Node>>, start: usize, targets: &Vec<usize>) -> Vec<usize> {
    let mut connected_targets = Vec::new();
    let mut heap = BinaryHeap::new();
    let mut visited = vec![false; graph.len()];

    heap.push(Node { distance: 0, index: start });

    while let Some(Node { index, distance }) = heap.pop() {
        if visited[index] {
            continue;
        }

        visited[index] = true;

        if targets.contains(&index) {
            connected_targets.push(index);
        }

        for &neighbor in graph[index].iter() {
            if !visited[neighbor.index] {
                heap.push(Node {
                    distance: distance + neighbor.index,
                    index: neighbor.index,
                });
            }
        }
    }
    connected_targets
}

/// Ottiene le coordinate di ogni elemento in una matrice e restituisce una mappa degli indici agli accoppiamenti (riga, colonna).
///
/// # Parametri
///
/// - `matrix`: La matrice rappresentata come un vettore di vettori di Tile.
///
/// # Restituisce
///
/// Una mappa contenente gli indici degli elementi nella matrice come chiavi e le rispettive coordinate (riga, colonna) come valori.
///
fn get_coordinates(matrix: &Vec<Vec<Tile>>) -> HashMap<usize, (usize, usize)>{
    let mut hm = HashMap::new();
    let mut current_index = 0;

    for i in 0..matrix.len() {
        for j in 0..matrix[i].len() {
            hm.insert(current_index,(i, j));
            current_index += 1;
        }
    }
    hm
}