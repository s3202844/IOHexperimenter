#pragma once
#include "ioh/problem/utils.hpp"
#include "pbo_problem.hpp"

namespace ioh
{
    namespace problem
    {
        namespace pbo
        {
            //! LeadingOnesDummy2 problem id 12
            class LeadingOnesDummy2 final: public PBOProblem<LeadingOnesDummy2>
            {
                std::vector<int> info_;
            protected:
              
                //! Evaluation method
                double evaluate(const std::vector<int> &x) override
                {
                    auto result = 0.0;
                    for (size_t i = 0; i != info_.size(); ++i)
                    {
                        if (x[info_[i]] == 1)
                            result = static_cast<double>(i) + 1.;
                        else
                            break;
                    }
                    return result;
                }

            public:
                /**
               * \brief Construct a new LeadingOnes_Dummy2 object. Definition refers to
               *https://doi.org/10.1016/j.asoc.2019.106027
               *
               * \param instance The instance number of a problem, which controls the transformation
               * performed on the original problem.
               * \param n_variables The dimensionality of the problem to created, 4 by default.
               **/
                LeadingOnesDummy2(const int instance, const int n_variables) :
                    PBOProblem(12, instance, n_variables, "LeadingOnesDummy2"),
                    info_(utils::dummy(n_variables, 0.9, 10000))
                {
                    optimum_.x = std::vector<int>(n_variables,1);
                    optimum_.y = evaluate(optimum_.x);
                    optimum_.x = reset_transform_variables(optimum_.x);
                    optimum_.y = transform_objectives(optimum_.y);
                }
            };
        } // namespace pbo
    } // namespace problem
} // namespace ioh
