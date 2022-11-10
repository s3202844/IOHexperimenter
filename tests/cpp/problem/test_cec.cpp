#include "../utils.hpp"

#include "ioh/problem/cec.hpp"

using namespace ioh::problem::cec;

TEST_F(BaseTest, loadMatrixData)
{
    CecData cd;
    std::string dataPath = "/home/ian/IOHexperimenter/static/cec_data";
    loadMatrixData(&cd, dataPath, 2, 1, 2022);
    for (auto d : cd.Mr)
    {
        std::cout << d << " ";
    }
    std::cout << std::endl;
}


TEST_F(BaseTest, loadOShiftData)
{
    CecData cd;
    std::string dataPath = "/home/ian/IOHexperimenter/static/cec_data";
    loadOShiftData(&cd, dataPath, 2, 1, 2022);
    for (auto d : cd.Os)
    {
        std::cout << d << " ";
    }
    std::cout << std::endl;
}

TEST_F(BaseTest, cec2022_01)
{
    // std::ofstream file;
    // file.open("/home/ian/cec_core/results/exp_data/F1.txt", std::ios::out);
    // if (!file.is_open())
    // {
    //     std::cout << "open failed" << std::endl;
    // }
    // CecData cd;
    // std::vector<double> x = {10, 10};
    // std::string dataPath = "/home/ian/IOHexperimenter/static/cec_data";
    // loadOShiftData(&cd, dataPath, 2, 1, 2022);
    // loadMatrixData(&cd, dataPath, 2, 1, 2022);

    auto problem = cec2022_01(1, 2);
    // problem.Os_ = cd.Os;
    // problem.Mr_ = cd.Mr;
    // for (double x1 = -100.0; x1 <= 100.0; x1 += 0.5)
    // {
    //     for (double x2 = -100.0; x2 <= 100.0; x2 += 0.5)
    //     {
    //         std::vector<double> x = {x1, x2};
    //         file << std::to_string(x1) << " " << std::to_string(x2) << " "
    //              << std::to_string(problem(x)) << std::endl;
    //     }
    // }
    // file.close();
}
