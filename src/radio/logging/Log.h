//
// Created by nicholasball on 1/24/23.
//

#include <filesystem>
#include <fstream>
#include <sys/stat.h>
#include <chrono>
#include <ctime>

#ifndef RADIO_LOGGING_H
#define RADIO_LOGGING_H



// Helper function to check if a file exists
inline bool exists(const std::string& name) {
    struct stat buffer;
    return (stat (name.c_str(), &buffer) == 0);
}

class Log
{
public:
    std::string What;

    /*!
     * Create and log a message
     *
     * @abstract Create and log a message
     *
     * @param   what  The logging message to record
     * @param   file  The file where the logging will be recorded
    */
    Log(std::string what, std::string file) : What(what)
    {
        Send_To_Log(file);
    }

    /*!
     * Send logging information to specified log file
     *
     * @function Send_To_Log
     *
     * @abstract Send logging information to log file
     *
     * @param   file  The file where the logging will be recorded
    */
    void Send_To_Log(std::string file)
    {
        try {
            // Get the directory of file, if one exists
            int index = file.find_last_of('/');
            std::string dir = "";
            if (index < (int) file.size()) dir = file.substr(0, index);

            // Get current time
            auto end = std::chrono::system_clock::now();
            std::time_t end_time = std::chrono::system_clock::to_time_t(end);
            std::string t = std::string(std::ctime(&end_time));

            // remove newline at end of ctime (Kind of odd it does that)
            t = t.substr(0, t.size() - 1);

            // Set time in which the log report is made
            std::string pre_message = std::string("[") + std::string(t) + std::string("] ");

            //if file does not exist, create it
            if (!exists(file)) {
                //create directory
                std::filesystem::create_directories(dir);

                // write the time in which the log is created
                std::ofstream outfile(file);
                outfile << pre_message + "Created log file" << std::endl;
                outfile.close();

            }

            // Send logging to log
            std::fstream open_file(file, std::ios_base::app);
            open_file << pre_message + What << std::endl;
            open_file.close();
        }catch (...)
        {
            std::cout<<"Error logging!"<<std::endl;
        }
    }
};
#endif //RADIO_LOGGING_H
