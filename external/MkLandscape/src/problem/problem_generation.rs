/*!
Module to generate problems (TD Mk Landscapes) using passed codomain, read these problems and write them (using (de)serialization ).
*/

use itertools::Itertools;
use rand_chacha::ChaChaRng;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

use std::{
    error::Error,
    fmt::Write as fmt_write,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};


use super::{
    clique_tree::{CliqueTree, InputParameters},
    codomain::{read_codomain, generate_write_return},
    io::{get_clique_tree_from_codomain_file, get_clique_trees_paths_from_codomain_folder, 
            get_output_folder_path_from_configuration_file},
    configuration::{get_rng}
};

use super::configuration::ConfigurationParameters;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "Problem Generator",
    about = "Generate TD Mk Landscape problem using a codomain file"
)]
pub struct ProblemOpt {
    #[structopt(subcommand)]
    pub problem_command: ProblemCommand,
    #[structopt(short = "s", long = "seed")]
    pub seed: Option<u64>,
}

#[derive(StructOpt, Debug)]
pub enum ProblemCommand {
    /// Generate problems for configurations specified in a given directory that contains a directory 'codomain_files'
    ///  with codomain files that specify both the topology and codomain
    #[structopt(name = "codomain_folder")]
    CodomainFolder {
        ///Input folder that contains a 'codomain_files' folder to read and to generate problems for
        #[structopt(parse(from_os_str))]
        folder_paths: Vec<PathBuf>,
        ///Whether the codomain was generated by the problem generator / whether the codomain contains the codomain function on the first line
        #[structopt(short = "g")]
        generated: bool,
    },
    /// Generate problems for configurations specified in a given directory that contains a directory 'problem_generation'
    ///  with files specifying the codomain parameters and ranges of topology parameters
    #[structopt(name = "configuration_folder")]
    ConfigurationFolder {
        ///Input path that contains a 'problem_generation' folder to read and to generate codomain files and problems for.
        #[structopt(parse(from_os_str))]
        folder_paths: Vec<PathBuf>,
        ///number of problems to generate per configuration instance
        #[structopt(default_value = "1", short = "n")]
        number_of_problems_to_generate: u32,
    },
    /// Generate problems for a configuration specified in a given file that already contains the codomain
    #[structopt(name = "codomain_file")]
    CodomainFile {
        ///Input codomain file to read and to generate problems for
        #[structopt(parse(from_os_str))]
        input_codomain_file_path: PathBuf,
        ///File to write the generated problem to
        #[structopt(parse(from_os_str))]
        output_problem_file_path: PathBuf,
        ///Whether the codomain was generated by the problem generator / whether the codomain contains the codomain function on the first line
        #[structopt(short = "g")]
        generated: bool,
    },
    /// Generate problems for ranges of configurations specified in a given file
    #[structopt(name = "configuration_file")]
    ConfigurationFile {
        ///Input configuration_parameters file to read and to generate codomain files and problems for
        #[structopt(parse(from_os_str))]
        input_configuration_file_path: PathBuf,
        ///File or folder to write the generated codomains to
        #[structopt(parse(from_os_str))]
        output_codomain_folder_path: PathBuf,
        ///File or folder to write the generated problem(s) to
        #[structopt(parse(from_os_str))]
        output_problem_folder_path: PathBuf,
        ///number of problems to generate per configuration instance
        #[structopt(default_value = "1", short = "n")]
        number_of_problems_to_generate: u32,
    },
}

///Run codomain generator from command line options (structopt)
pub fn run_opt(problem_opt: ProblemOpt) -> Result<(), Box<dyn Error>> {
    let mut rng = get_rng(problem_opt.seed);
    match problem_opt.problem_command {
        ProblemCommand::CodomainFolder {
            folder_paths,
            generated,
        } => {
            for folder_path in folder_paths {
                generate_problems_from_codomain_folder(&folder_path, generated, &mut rng)?;
            }
            Ok(())
        }
        ProblemCommand::ConfigurationFolder {
            folder_paths,
            number_of_problems_to_generate,
        } => {
            for folder_path in folder_paths {
                generate_codomain_and_problem_from_folder(
                    &folder_path,
                    number_of_problems_to_generate,
                    &mut rng,
                )?;
            }
            Ok(())
        }
        ProblemCommand::CodomainFile {
            input_codomain_file_path,
            output_problem_file_path,
            generated,
        } => {
            generate_problem_from_codomain_file(
                &input_codomain_file_path,
                &output_problem_file_path,
                generated,
                &mut rng
            )
        },
        ProblemCommand::ConfigurationFile {
            input_configuration_file_path,
            output_codomain_folder_path,
            output_problem_folder_path,
            number_of_problems_to_generate,
        } => {
            generate_codomain_and_problem(
                &input_configuration_file_path,
                Some(&output_codomain_folder_path),
                Some(&output_problem_folder_path),
                number_of_problems_to_generate,
                &mut rng
            )
        }
    }
}

///Structure to store a generated problem for writing to a file
/// The difference with the CliqueTree structure is the exclusion of the codomain values and function (as these are stored separately)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Problem {
    pub input_parameters: InputParameters,
    pub glob_optima_score: f64,
    pub glob_optima_strings: Vec<Vec<u32>>,
    pub cliques: Vec<Vec<u32>>,
}

impl Problem {
    fn new(clique_tree: &CliqueTree) -> Problem {
        Problem {
            input_parameters: clique_tree.input_parameters.clone(),
            cliques: clique_tree.cliques.clone(),
            glob_optima_score: clique_tree.glob_optima_score,
            glob_optima_strings: clique_tree.glob_optima_strings.clone(),
        }
    }
}

///Generate problems from the codomain and input parameters,
/// which are both given by the files in the parent's codomain folder and write them to the parent's problems folder
pub fn generate_problems_from_codomain_folder(
    parent_folder_path: &Path,
    generated: bool,
    rng: &mut ChaChaRng
) -> Result<(), Box<dyn Error>> {
    let mut codomain_folder_path = PathBuf::from(parent_folder_path);
    codomain_folder_path.push("codomain_files");
    let mut problems_folder_path = PathBuf::from(parent_folder_path);
    problems_folder_path.push("problems");

    //get all folder entries in the codomain_files folder
    let folder_entries: Vec<PathBuf> = codomain_folder_path
        .read_dir()?
        .map(|folder| folder.unwrap().path())
        .sorted()
        .collect();

    //For each folder f,
    for folder in folder_entries {
        // Create a directory in the problems folder with the same name (f)
        let mut output_folder_path = problems_folder_path.clone();
        output_folder_path.push(
            folder
                .file_name()
                .ok_or("could not get file name of folder")?,
        );
        std::fs::create_dir_all(&output_folder_path)?;

        //And generate problems / clique trees for all codomain files in the codomain folder f
        let clique_trees_paths = get_clique_trees_paths_from_codomain_folder(&folder, generated, rng)?;
        for (clique_tree, path_buf) in clique_trees_paths {
            let mut output_path = output_folder_path.clone();
            output_path.push(
                path_buf
                    .file_name()
                    .ok_or("could not get filename of codomain file")?,
            );
            //write the output problems to disk
            write_problem_to_file(&clique_tree, &output_path)?;
        }
    }
    Ok(())
}

///Generate a problem from the codomain and input parameters given by codomain_file_path and write it to output_path
pub fn generate_codomain_and_problem_from_folder(
    input_folder_path: &Path,
    number_of_problems_to_generate: u32,
    rng: &mut ChaChaRng
) -> Result<(), Box<dyn Error>> {
    //Use the input_folder_path to get the problem_generation folder and problems folder paths
    let mut problem_generation_path = PathBuf::from(input_folder_path);
    problem_generation_path.push("problem_generation");

    //For each file in the problem_generation folder,
    let file_entries: Vec<PathBuf> = problem_generation_path
        .read_dir()?
        .map(|file| file.unwrap().path())
        .sorted()
        .collect();

    // generate all codomain and problem files and write them to the codomain_files and problems folders
    for file in file_entries {
        generate_codomain_and_problem(&file, None, None, number_of_problems_to_generate, rng)?;
    }
    Ok(())
}

///Generate codomain and problem files for the input configuration as read from the input_configuration_file.
/// If the output_(codomain/problem)_folder_path is None, we default to folder paths used in other parts of the program (codomain_files & problems).
/// If they are Some(path), we use the path as the destination folder.
pub fn generate_codomain_and_problem(
    input_configuration_file_path: &Path,
    output_codomain_folder_path: Option<&Path>,
    output_problem_folder_path: Option<&Path>,
    number_of_problems_to_generate: u32,
    rng: &mut ChaChaRng
) -> Result<(), Box<dyn Error>> {
    //Get the configuration parameters from the input configuration file
    let configuration_parameters =
        ConfigurationParameters::from_file(input_configuration_file_path)?;

    let codomain_function = configuration_parameters.codomain_function.clone();

    //if an output_problem_folder_path is passed, we use it, otherwise we default to our way of calculating where the file should go (into problems folder)
    let output_problem_folder_path_buf = match output_problem_folder_path {
        Some(folder) => PathBuf::from(folder),
        None => get_output_folder_path_from_configuration_file(
            input_configuration_file_path,
            "problems",
        )?,
    };

    //if a output_codomain_folder_path is passed, we use it, otherwise we default to our way of calculating where the file should go (into codomain_files)
    let output_codomain_folder_path_buf = match output_codomain_folder_path {
        Some(folder) => PathBuf::from(folder),
        None => get_output_folder_path_from_configuration_file(
            input_configuration_file_path,
            "codomain_files",
        )?,
    };

    //Loop over all input parameters (using custom iterator)
    for input_parameters in configuration_parameters {
        //Generate number_problems different problem instances for each input parameter configuration
        for num in 0..number_of_problems_to_generate {
            let mut output_problem_file_path = output_problem_folder_path_buf.clone();
            let mut output_codomain_file_path = output_codomain_folder_path_buf.clone();

            let output_file_name = format!(
                "{}_{}_{}_{}_{}_{}.txt",
                codomain_function.to_io_string(),
                input_parameters.m,
                input_parameters.k,
                input_parameters.o,
                input_parameters.b,
                num
            );

            output_problem_file_path.push(output_file_name.clone());
            output_codomain_file_path.push(output_file_name);
            //println!("constructed output file path: {:?}", output_file_path);

            let codomain =
                generate_write_return(&input_parameters, &codomain_function, &output_codomain_file_path, rng)?;

            //Generate a clique tree using the input parameter, the codomain function, and the codomain values
            let clique_tree = CliqueTree::new(
                input_parameters.clone(),
                codomain_function.clone(),
                codomain,
                rng
            );

            //Write the problem to disk
            write_problem_to_file(&clique_tree, &output_problem_file_path)?;
        }
    }
    Ok(())
}

///Generate a problem from the codomain and input parameters given by codomain_file_path and write it to the problem at the output_problem path
pub fn generate_problem_from_codomain_file(
    codomain_file_path: &Path,
    output_problem_file_path: &Path,
    generated: bool,
    rng: &mut ChaChaRng
) -> Result<(), Box<dyn Error>> {
    //Get the clique tree from the codomain file
    let clique_tree = get_clique_tree_from_codomain_file(codomain_file_path, generated, rng)?;
    //Write the problem to file
    write_problem_to_file(&clique_tree, output_problem_file_path)
}

///Read the clique tree from the problem and codomain values, from the problem file and codomain file
pub fn read_clique_tree_from_files(
    problem_path: &Path,
    codomain_path: &Path,
    generated: bool,
) -> Result<CliqueTree, Box<dyn Error>> {
    let problem = read_problem_from_file(problem_path)?;
    let skip_lines = if generated { 2 } else { 1 };
    let codomain = read_codomain(&problem.input_parameters, codomain_path, skip_lines)?;
    Ok(CliqueTree::construct_from_problem_codomain(
        problem, codomain,
    ))
}

///Read the TD Mk Landscapes / clique trees from the codomain and problem folders.
/// We return a Vector of tuples that contain both the clique tree and the path to the codomain file.
///  The path is required to construct the output file path.
pub fn read_clique_trees_paths_from_folders(
    codomain_folder_path: &Path,
    problem_folder_path: &Path,
    generated: bool,
) -> Result<Vec<(CliqueTree, PathBuf)>, Box<dyn Error>> {
    //Get all codomain files
    let codomain_file_entries: Vec<PathBuf> = codomain_folder_path
        .read_dir()?
        .map(|file| file.unwrap().path())
        .sorted()
        .collect();
    //Get all problem files
    let problem_file_entries: Vec<PathBuf> = problem_folder_path
        .read_dir()?
        .map(|file| file.unwrap().path())
        .sorted()
        .collect();

    assert_eq!(codomain_file_entries.len(), problem_file_entries.len());

    let mut result_vec = Vec::new();

    //zip the (sorted) codomains and problems, and read the clique tree from the codomain and problem files.
    for (codomain_file_entry, problem_file_entry) in codomain_file_entries
        .into_iter()
        .zip(problem_file_entries.into_iter())
    {
        //Construct tuple from the read clique tree and the codomain file path
        result_vec.push((
            read_clique_tree_from_files(&problem_file_entry, &codomain_file_entry, generated)?,
            codomain_file_entry,
        ));
    }

    Ok(result_vec)
}

/// Write problem to file, for possible later use
pub fn write_problem_to_file(
    clique_tree: &CliqueTree,
    output_problem_file_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(output_problem_file_path)?;
    let mut buf_writer = BufWriter::new(file);
    let mut write_buffer = String::new();

    //Write the input parameters on the first line
    writeln!(
        write_buffer,
        "{} {} {} {}",
        clique_tree.input_parameters.m,
        clique_tree.input_parameters.k,
        clique_tree.input_parameters.o,
        clique_tree.input_parameters.b
    )?;
    buf_writer.write_all(write_buffer.as_bytes())?;
    write_buffer.clear();

    //Global optima fitness and solutions:
    //      fitness
    //      number_of_solutions
    //      solutions

    //fitness
    writeln!(write_buffer, "{}", clique_tree.glob_optima_score)?;
    buf_writer.write_all(write_buffer.as_bytes())?;
    write_buffer.clear();

    //number_of_solutions
    writeln!(write_buffer, "{}", clique_tree.glob_optima_strings.len())?;
    buf_writer.write_all(write_buffer.as_bytes())?;
    write_buffer.clear();

    //solutions
    for sol in &clique_tree.glob_optima_strings {
        for bit in sol {
            write!(write_buffer, "{}", bit)?;
        }
        writeln!(write_buffer)?;
    }
    buf_writer.write_all(write_buffer.as_bytes())?;
    write_buffer.clear();

    //Cliques/Subfunctions
    //      Per clique; variable indices
    for clique in &clique_tree.cliques {
        for variable_index in clique {
            write!(write_buffer, "{} ", variable_index)?;
        }
        write_buffer.pop().ok_or(
            "could not remove trailing white space from clique indices while writing problem",
        )?;
        writeln!(write_buffer)?;
    }
    buf_writer.write_all(write_buffer.as_bytes())?;
    write_buffer.clear();

    buf_writer.flush()?;

    Ok(())
}

///Write problem to file using serialization
pub fn write_problem_to_file_ser(
    clique_tree: &CliqueTree,
    file_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(file_path)?;
    let mut buf_writer = BufWriter::new(file);
    let mut write_buffer = String::new();

    let problem = Problem::new(clique_tree);

    //Write problem to file
    let my_config = ron::ser::PrettyConfig::new().with_depth_limit(4);
    let string =
        ron::ser::to_string_pretty(&problem, my_config).map_err(|_| "Serialization error!")?;

    write!(write_buffer, "{}", string)?;
    buf_writer.write_all(write_buffer.as_bytes())?;
    write_buffer.clear();

    buf_writer.flush()?;

    Ok(())
}

///Read problem from file
pub fn read_problem_from_file(file_path: &Path) -> Result<Problem, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut content_iter = reader.lines();

    //Read input parameters
    let mut line = content_iter.next().ok_or("Empty problem file")??;
    let parameters: Vec<&str> = line.split(' ').collect();
    if parameters.len() != 4 {
        return Err("not enough input parameters on first line of input file".into());
    }
    //And set the parameters
    let m: u32 = parameters[0].parse()?;
    let k: u32 = parameters[1].parse()?;
    let o: u32 = parameters[2].parse()?;
    let b: u32 = parameters[3].parse()?;

    let input_parameters = InputParameters::new_from_primitives(m, k, o, b);

    let problem_size = (m - 1) * (k - o) + k;

    //Read global optmium score
    line = content_iter
        .next()
        .ok_or("No global optimum score in problem file")??;
    let glob_optima_score: f64 = line.parse()?;

    //Read number_of_global_optima
    line = content_iter
        .next()
        .ok_or("No number_of_global_optima line in problem file")??;
    let number_of_global_optima: usize = line.parse()?;

    //Read global optima
    let mut glob_optima_strings = Vec::with_capacity(number_of_global_optima);
    for _i in 0..number_of_global_optima {
        line = content_iter
            .next()
            .ok_or("Not enough global optima strings in problem file")??;
        let mut chars = line.chars();
        let mut global_optimum: Vec<u32> = Vec::with_capacity(problem_size as usize);
        for _j in 0..problem_size as usize {
            let bit = chars
                .next()
                .ok_or("global optimum in problem file does not contain enough bits")?;
            global_optimum.push(
                bit.to_digit(10)
                    .ok_or("Could not convert global optimum bit from char to u32")?,
            );
        }
        glob_optima_strings.push(global_optimum);
    }

    //Read clique_tree cliques
    let mut cliques = Vec::with_capacity(m as usize);
    for _i in 0..m as usize {
        line = content_iter
            .next()
            .ok_or("Not enough cliques in problem file")??;
        let variable_indices: Vec<&str> = line.split(' ').collect();
        if variable_indices.len() != k as usize {
            return Err("not enough variable indices in clique indices".into());
        }
        let mut clique_indices: Vec<u32> = Vec::with_capacity(k as usize);
        for j in 0..k as usize {
            clique_indices.push(variable_indices[j].parse()?);
        }
        cliques.push(clique_indices);
    }

    let problem = Problem {
        input_parameters,
        glob_optima_score,
        glob_optima_strings,
        cliques,
    };

    Ok(problem)
}

///Read problem from file using deserialization
pub fn read_problem_from_file_de(file_path: &Path) -> Result<Problem, Box<dyn Error>> {
    let f = File::open(file_path)?;
    let mut reader = BufReader::new(f);
    let problem = ron::de::from_reader(&mut reader)?;
    Ok(problem)
}