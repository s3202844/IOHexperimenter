#pragma once

#include "ioh/problem/problem.hpp"
#include "ioh/problem/transformation.hpp"

namespace ioh::problem
{
        
    
    
    
    //! BBOB base class
    class BBOB : public Real
    {
    protected:
        /**
         * @brief Container for BBOB transformation data
         * 
         */
        struct TransformationState
        {
            //! The seed
            long seed;

            //! A vector with exponents
            std::vector<double> exponents{};

            //! A vector with conditions
            std::vector<double> conditions{};
            
            //! Main transformation matrix
            std::vector<std::vector<double>> transformation_matrix{};
            
            //! Main transformation vector
            std::vector<double> transformation_base{};

            //! Second transformation matrix
            std::vector<std::vector<double>> second_transformation_matrix{};

            //! First rotation matrix
            std::vector<std::vector<double>> first_rotation{};

            //! Second rotation matrix
            std::vector<std::vector<double>> second_rotation{};

            /**
             * @brief Construct a new Transformation State object
             * 
             * @param problem_id the id of the problem
             * @param instance the instance of the problem
             * @param n_variables the dimension of the problem
             * @param condition the conditioning of the problem
             */
            TransformationState(const long problem_id, const int instance, const int n_variables,
                                const double condition = sqrt(10.0)) :
                seed((problem_id == 4 || problem_id == 18 ? problem_id - 1 : problem_id) + 10000 * instance),
                exponents(n_variables),
                conditions(n_variables),
                transformation_matrix(n_variables, std::vector<double>(n_variables)),
                transformation_base(n_variables),
                second_transformation_matrix(n_variables, std::vector<double>(n_variables)),
                first_rotation(compute_rotation(seed + 1000000, n_variables)),
                second_rotation(compute_rotation(seed, n_variables))
            {
                for (auto i = 0; i < n_variables; ++i)
                    exponents[i] = static_cast<double>(i) / (static_cast<double>(n_variables) - 1);

                transformation_matrix = first_rotation;

                for (auto i = 0; i < n_variables; ++i)
                    for (auto j = 0; j < n_variables; ++j)
                        for (auto k = 0; k < n_variables; ++k)
                            second_transformation_matrix[i][j] += first_rotation.at(i).at(k)
                                * pow(condition, exponents.at(k))
                                * second_rotation.at(k).at(j);
            }

            /**
             * @brief Compute a rotation for a problem
             * 
             * @param rotation_seed the seed of the rotation
             * @param n_variables the dimension of the problem
             * @return std::vector<std::vector<double>> the rotation
             */
            [[nodiscard]]
            std::vector<std::vector<double>> compute_rotation(const long rotation_seed, const int n_variables) const
            {
                const auto random_vector = common::random::bbob2009::normal(static_cast<size_t>(n_variables) * n_variables, rotation_seed);
                auto matrix = std::vector<std::vector<double>>(n_variables, std::vector<double>(n_variables));

                // reshape
                for (auto i = 0; i < n_variables; i++)
                    for (auto j = 0; j < n_variables; j++)
                        matrix[i][j] = random_vector.at(static_cast<size_t>(j) * n_variables + i);


                /*1st coordinate is row, 2nd is column.*/
                for (long i = 0; i < n_variables; i++)
                {
                    for (long j = 0; j < i; j++)
                    {
                        auto prod = 0.0;
                        for (auto k = 0; k < n_variables; k++)
                            prod += matrix[k][i] * matrix[k][j];

                        for (auto k = 0; k < n_variables; k++)
                            matrix[k][i] -= prod * matrix[k][j];
                    }
                    auto prod = 0.0;
                    for (auto k = 0; k < n_variables; k++)
                        prod += matrix[k][i] * matrix[k][i];

                    for (auto k = 0; k < n_variables; k++)
                        matrix[k][i] /= sqrt(prod);
                }
                return matrix;
            }
        } 
        //! The current transformation state
        transformation_state_;

        //! Default objective transform for BBOB
        double transform_objectives(const double y) override
        {
            return transformation::objective::shift(y, optimum_.y);
        }

    public:
        /**
         * @brief Construct a new BBOB object
         * 
         * @param problem_id The id of the problem
         * @param instance The instance of the problem
         * @param n_variables the dimension of the problem
        * @param name the name of the problem
         * @param condition the conditioning of the problem
         */
        BBOB(const int problem_id, const int instance, const int n_variables, const std::string &name,
             const double condition = sqrt(10.0)):
            Real(MetaData(problem_id, instance, name, n_variables),
                 Bounds<double>(n_variables, -5, 5)),
            transformation_state_(problem_id, instance, n_variables, condition)
        {
            optimum_ = calculate_objective();
            log_info_.optimum = optimum_;
        }

        //! Calculate the solution to the problem
        [[nodiscard]]
        Solution<double> calculate_objective() const
        {
            using namespace common::random::bbob2009;

            auto x = uniform(meta_data_.n_variables,
                             transformation_state_.seed + (1000000 * (meta_data_.problem_id == 12)));

            for (auto &xi : x)
            {
                xi = 8 * floor(1e4 * xi) / 1e4 - 4;
                if (xi == 0.0)
                    xi = -1e-5;
            }

            const auto r1 = normal(1, transformation_state_.seed).at(0);
            const auto r2 = normal(1, transformation_state_.seed + 1).at(0);

            return {x, std::min(1000., std::max(-1000., floor((100. * 100. * r1 / r2) + 0.5) / 100.))};
        }
    };
    
    /**
     * @brief CRTP class for BBOB problems. Inherit from this class when defining new BBOB problems
     * 
     * @tparam ProblemType The New BBOB problem class
     */
    template <typename ProblemType>
    class BBOProblem : public BBOB,
                       AutomaticProblemRegistration<ProblemType, BBOB>,
                       AutomaticProblemRegistration<ProblemType, Real>
    {
    public:
        /**
         * @brief Construct a new BBOProblem object
         * 
         * @param problem_id The id of the problem
         * @param instance The instance of the problem
         * @param n_variables the dimension of the problem
         * @param name the name of the problem
         * @param condition the conditioning of the problem
         */
        BBOProblem(const int problem_id, const int instance, const int n_variables, const std::string &name,
                   const double condition = sqrt(10.0)) :
            BBOB(problem_id, instance, n_variables, name, condition)
        {
        }
    };
}
