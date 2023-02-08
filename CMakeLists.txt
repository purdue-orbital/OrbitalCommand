cmake_minimum_required(VERSION 3.20)

#------------------
# Break up projects
#------------------

#------------------
# main projects
#------------------
project(ground_station)
project(launch_station)

#------------------
# Test projects
#------------------
project(radio_modulation)

set(CMAKE_CXX_STANDARD 20)

# include library
find_library(LIME LimeSuite lib)


# add executables
add_executable(ground_station src/ground_station/ground_station.cpp)
add_executable(launch_station src/launch_station/launch_station.cpp)
add_executable(radio_modulation test/radio_modulation.cpp)

