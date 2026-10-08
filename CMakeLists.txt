cmake_minimum_required(VERSION 3.16)

project(tic VERSION 0.1 LANGUAGES CXX RC)

set(EXECUTABLE_OUTPUT_PATH ./${CMAKE_BUILD_TYPE})

add_definitions(-DUNICODE)

set(INCLUDE_FILES
    camera.hpp
    winapp.hpp
    plotter.hpp
    dxf.hpp
)

set(SRC_FILES
    main.cpp
    camera.cpp
    winapp.cpp
    plotter.cpp
    dxf.cpp
)

list(TRANSFORM INCLUDE_FILES PREPEND ${CMAKE_CURRENT_SOURCE_DIR}/include/)
list(TRANSFORM SRC_FILES PREPEND ${CMAKE_CURRENT_SOURCE_DIR}/src/)

add_executable(bcut ${INCLUDE_FILES} ${SRC_FILES} res/resource.rc)

target_include_directories(bcut PRIVATE 
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${CMAKE_CURRENT_SOURCE_DIR}/vendor/dxfrw/include
    ${CMAKE_CURRENT_SOURCE_DIR}/vendor/tinyspline/include
    )

if(NOT CMAKE_BUILD_TYPE STREQUAL "Debug")
    set_property(TARGET bcut PROPERTY MSVC_RUNTIME_LIBRARY "MultiThreaded")
    set(LIB_FOLDER lib_release)
else()
    set(LIB_FOLDER lib)
endif()

target_link_directories(bcut PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/vendor/dxfrw/${LIB_FOLDER}
        ${CMAKE_CURRENT_SOURCE_DIR}/vendor/tinyspline/${LIB_FOLDER}
)

target_link_libraries(bcut
    Gdiplus
    dxfrw
    tinyspline
    tinysplinecxx
    Shlwapi
)

target_link_options(bcut PRIVATE -SUBSYSTEM:WINDOWS)