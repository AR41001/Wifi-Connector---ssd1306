#![allow(unused)]
use embedded_graphics::{
    fonts::{Font6x12, Text},
    pixelcolor::BinaryColor,
    prelude::*,
    style::TextStyleBuilder,
};
use linux_embedded_hal::I2cdev;
use nanomsg::{Error, Protocol, Socket};
use rppal::gpio::Gpio;
use ssd1306::{mode::GraphicsMode, Builder, I2CDIBuilder};
use std::fs::{self, File};
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::process::Command;
use std::string::String;
use std::sync::mpsc;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use std::vec::Vec;
use std::{fs::OpenOptions, time::Instant};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::borrow::Cow;
use std::sync::atomic::{AtomicBool, Ordering};

// Define the number of lines to display on the screen at a time
const LINES_PER_SCREEN: usize = 5;              // can be any number you want



fn main() {

    let temp_password_elements: [char; 94] = [
        '_', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q',
        'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
        'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0',
        '1', '2', '3', '4', '5', '6', '7', '8', '9', '!', '@', '#', '$', '%', '^', '&', '*', '(',
        ')', '-', '_', '=', '+', '{', '}', '[', ']', '~', '`', ':', ';', '"', '>', '<', ',', '.',
        '/', '?', '|', '\\',
    ];
    let main_menu_elements: [&str; 1] = ["Wifi Settings"];

    let mut password_file = OpenOptions::new()
        .append(true)
        .open("/home/lab/Documents/oled_screen/target/release/passwords.txt")
        .expect("cannot open file");

    let mut pass_array: Vec<char> = Vec::new();
    let gpio = Gpio::new().unwrap();
    let scroll_down_button = gpio.get(26).unwrap().into_input_pullup();
    let scroll_up_button = gpio.get(27).unwrap().into_input_pullup(); //pin number to be decided
    let select_button = gpio.get(25).unwrap().into_input_pullup();
    let back_button = gpio.get(6).unwrap().into_input_pullup();
    let reset_button = gpio.get(22).unwrap().into_input_pullup(); //pin number to be decided

    let i2c = I2cdev::new("/dev/i2c-1").unwrap();
    let interface = I2CDIBuilder::new().init(i2c);
    let mut disp: GraphicsMode<_> = Builder::new().connect(interface).into();

    disp.init().unwrap();
    disp.flush().unwrap();

    let text_style_main = TextStyleBuilder::new(Font6x12)
        .text_color(BinaryColor::On) // binary off = black
        .background_color(BinaryColor::Off) // binary on  = blue
        .build();

    let text_style_selected = TextStyleBuilder::new(Font6x12)
        .text_color(BinaryColor::Off) // binary off = black
        .background_color(BinaryColor::On) // binary on  = blue
        .build();

    // Get all WiFi SSID names
    let mut all_ssids: Vec<String> = get_wifi_ssids();
    let mut selected_index = 0; // Track the currently selected index
    let mut start_index = 0; // Track the start index of displayed SSIDs
    let mut select_button_state = 0;
    let mut previous_select_button_state = 0;
    let mut _scroll_down_button_state = 0;
    let mut previous_scroll_down_button_state = 0;
    let mut _scroll_up_button_state = 0;
    let mut previous_scroll_up_button_state = 0;
    let mut _reset_button_state = 0;
    let mut _back_button_state = 0;
    let mut previous_back_button_state = 0;
    let mut double_click_state = 0;
    let mut _save_password_state = 0;
    let mut end_index = start_index + LINES_PER_SCREEN - 1;
    let mut menu_state = 0;
    let mut ssid_index = 0;
    let mut _menu_length = 0;
    let mut char_index = 0;
    let mut _pass_index = 0;

 
    let mut password_check: &str = "_";
    let mut temp_password = String::from("");
    let mut temp_pin = String::from("_");
    let mut temp_ip = String::from("_");
    let mut password = String::from("_");
    
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    ctrlc::set_handler(move || {
    running_clone.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    while running.load(Ordering::SeqCst) {
    if menu_state == 1 {
            for (i, element) in main_menu_elements
                .iter()
                .skip(start_index)
                .take(LINES_PER_SCREEN)
                .enumerate()
            {
                if i == selected_index {
                    let _ = Text::new(element, Point::new(0, i as i32 * 12))
                        .into_styled(text_style_selected.clone())
                        .draw(&mut disp);
                } else {
                    let _ = Text::new(element, Point::new(0, i as i32 * 12))
                        .into_styled(text_style_main.clone())
                        .draw(&mut disp);
                }
            }
        } else if menu_state == 2 {
          
             all_ssids = get_wifi_ssids();
            _menu_length = all_ssids.len();
            for (i, ssid) in all_ssids
                .iter()
                .skip(start_index)
                .take(LINES_PER_SCREEN)
                .enumerate()
            {
                if i == selected_index {
                    let _ = Text::new(ssid, Point::new(0, i as i32 * 12))
                        .into_styled(text_style_selected.clone())
                        .draw(&mut disp);
                } else {
                    let _ = Text::new(ssid, Point::new(0, i as i32 * 12))
                        .into_styled(text_style_main.clone())
                        .draw(&mut disp);
                }
            }
        }  else if menu_state == 5 {
            _menu_length = 2;
            ssid_index = selected_index + start_index;
            let _ = Text::new(all_ssids[ssid_index].as_str(), Point::new(0, 0))
                .into_styled(text_style_main.clone())
                .draw(&mut disp);
            let _ = Text::new("Enter password", Point::new(0, 12))
                .into_styled(text_style_selected.clone())
                .draw(&mut disp);
        } else if menu_state == 6 {
            ssid_index = selected_index + start_index;
            let _ = Text::new(all_ssids[ssid_index].as_str(), Point::new(0, 0))
                .into_styled(text_style_main.clone())
                .draw(&mut disp);
            let _ = Text::new("temp_password:", Point::new(0, 12))
                .into_styled(text_style_selected.clone())
                .draw(&mut disp);
            let _ = Text::new(temp_password.as_str(), Point::new(0, 30))
                .into_styled(text_style_main.clone())
                .draw(&mut disp);
        } else if menu_state == 7 {
            let _ = Text::new("PASSWORD ENTERED", Point::new(0, 0))
                .into_styled(text_style_selected.clone())
                .draw(&mut disp);
        } 

        disp.flush().unwrap();
        //--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
       if menu_state == 1 {
          
            while !scroll_down_button.is_high() {
                //this while loop is responsible for detecting button press
                if previous_scroll_down_button_state == 0 {
                 
                    _scroll_down_button_state = 1;

                    if selected_index == LINES_PER_SCREEN - 1 && _scroll_down_button_state == 1 {
                        // end of page algorithm to scroll the page when selected index is at the bottom as we press the button
                        start_index += 1;
                        end_index += 1;
                    } else if _scroll_down_button_state == 1 {
                        selected_index += 1;
                    }

                    if end_index >= all_ssids.len() {
                        //algo to come back to the first after end is reached
                        selected_index = 0; // Restart from the beginning
                        start_index = 0;
                        end_index = start_index + LINES_PER_SCREEN - 1;
                    }
                    _scroll_down_button_state = 0;
                }
                previous_scroll_down_button_state = 1;
            }
            while scroll_down_button.is_high() && previous_scroll_down_button_state == 1 {
                previous_scroll_down_button_state = 0;
            }
            while !select_button.is_high() {
                //this is to select any SSID and move on to the next menu
                if previous_select_button_state == 0 {
                  
                    select_button_state = 1;
                    if select_button_state == 1 {
                        if selected_index == 0 {
                            menu_state = 2;
                            start_index = 0;
                            selected_index = 0;
                            end_index = 0;
                        } else if selected_index == 1 {
                            menu_state = 3;
                            start_index = 0;
                            selected_index = 0;
                            end_index = 0;
                        } else if selected_index == 2 {
                            menu_state = 4;
                            start_index = 0;
                            selected_index = 0;
                            end_index = 0;
                        }
                    }
                    select_button_state = 0;
                }
                previous_select_button_state = 1;
               
            }
            while select_button.is_high() && previous_select_button_state == 1 {
                previous_select_button_state = 0;
            }
            while !scroll_up_button.is_high() {
                if previous_scroll_up_button_state == 0 {
             
                    _scroll_up_button_state = 1;

                    if _scroll_up_button_state == 1 && selected_index == 0 && start_index == 0 {
                    } else if _scroll_up_button_state == 1 && selected_index == 0 {
                        start_index -= 1;
                        end_index -= 1;
                    } else if _scroll_up_button_state == 1 && selected_index != 0 {
                        selected_index -= 1;
                    }

                    _scroll_up_button_state = 0;
                }
                previous_scroll_up_button_state = 1;
               
            }
            while scroll_up_button.is_high() && previous_scroll_up_button_state == 1 {
                previous_scroll_up_button_state = 0;
            }

            while !back_button.is_high() {
                if previous_back_button_state == 0 {
              
                    // the back button is obv used to go back to the previous menu
                    _back_button_state = 1;
                    if _back_button_state == 1 {
                        menu_state = 0;
                    }
                    _back_button_state = 0;
                }
                previous_back_button_state = 1;
                
            }
            while back_button.is_high() && previous_back_button_state == 1 {
                previous_back_button_state = 0;
            }
        } else if menu_state == 2 {

            //this is the main menu where all SSID's are displayed
            while !scroll_down_button.is_high() {
                //this while loop is responsible for detecting button press
                if previous_scroll_down_button_state == 0 {
     
                    _scroll_down_button_state = 1;

                    if selected_index == LINES_PER_SCREEN - 1 && _scroll_down_button_state == 1 {
                        // end of page algorithm to scroll the page when selected index is at the bottom as we press the button
                        start_index += 1;
                        end_index += 1;
                    } else if _scroll_down_button_state == 1 {
                        selected_index += 1;
                    }

                    if end_index >= all_ssids.len() {
                        //algo to come back to the first after end is reached
                        selected_index = 0; // Restart from the beginning
                        start_index = 0;
                        end_index = start_index + LINES_PER_SCREEN - 1;
                    }
                    _scroll_down_button_state = 0;
                }
                previous_scroll_down_button_state = 1;
             
            }

            while scroll_down_button.is_high() && previous_scroll_down_button_state == 1 {
                previous_scroll_down_button_state = 0;
            }
            while !select_button.is_high() {
                //this is to select any SSID and move on to the next menu
                if previous_select_button_state == 0 {
               
                    select_button_state = 1;
                    if select_button_state == 1 {
                        menu_state = 5;
                    }
                    select_button_state = 0;
                }
                previous_select_button_state = 1;
                
            }

            while select_button.is_high() && previous_select_button_state == 1 {
                previous_select_button_state = 0;
            }

            while !scroll_up_button.is_high() {
                if previous_scroll_up_button_state == 0 {
       
                    _scroll_up_button_state = 1;

                    if _scroll_up_button_state == 1 && selected_index == 0 && start_index == 0 {
                    } else if _scroll_up_button_state == 1 && selected_index == 0 {
                        start_index -= 1;
                        end_index -= 1;
                    } else if _scroll_up_button_state == 1 && selected_index != 0 {
                        selected_index -= 1;
                    }

                    _scroll_up_button_state = 0;
                }
                previous_scroll_up_button_state = 1;
       
            }

            while scroll_up_button.is_high() && previous_scroll_up_button_state == 1 {
                previous_scroll_up_button_state = 0;
            }

            while !back_button.is_high() {
                // the back button is obv used to go back to the previous menu
                if previous_back_button_state == 0 {
          
                    _back_button_state = 1;
                    if _back_button_state == 1 {
                        menu_state = 1;
                    }
                    _back_button_state = 0;
                }
                previous_back_button_state = 1;
      
            }

            while back_button.is_high() && previous_back_button_state == 1 {
                previous_back_button_state = 0;
            }
        } else if menu_state == 5 {

            //this is a useless menu made by shams, it has no point whatsoever, its just there
            while !select_button.is_high() {
                //here the select button just moves on to the next menu
                if previous_select_button_state == 0 {

                    select_button_state = 1;
                    if select_button_state == 1 {
                        menu_state = 6;
                    }
                    select_button_state = 0;
                }
                previous_select_button_state == 1;
            
            }
            while select_button.is_high() && previous_select_button_state == 1 {
                previous_select_button_state = 0;
            }
            while !back_button.is_high() {
                // the back button is obv used to go back to the previous menu
                if previous_back_button_state == 0 {
       
                    _back_button_state = 1;
                    if _back_button_state == 1 {
                        println!("changing to state 4");
                        menu_state = 2;
                    }
                   _back_button_state = 0;
                }
                previous_back_button_state = 1;
       
            }
            while back_button.is_high() && previous_back_button_state == 1 {
                previous_back_button_state = 0;
            }
        } else if menu_state == 6 {

            //this state is responsible for entering and storing the temp_password for any SSID

            /* Useable characters are stored in an array, a vector is initialized to store the temp_password as it is being entered.
               Scroll button cycles through characters and select button selects them.
               Final temp_password is stored in a string that we can use else where : temp_password
            */

            while !scroll_down_button.is_high() {
                if previous_scroll_down_button_state == 0 {

                    if select_button_state == 1 {
                        //If a character is selected it is pushed onto the vector array here
                        pass_array.push(temp_password_elements[char_index]);
                        select_button_state = 0;
                    }
                    char_index += 1;
                    if char_index == 94 {
                        //if last character reached, start again from the first character
                        char_index = 1;
                    }

                    if pass_array.len() > 0 {
                        // replaces the display character every cycle to make sure the latest character is being displayed
                        pass_array.remove(pass_array.len() - 1);
                    }
                    pass_array.push(temp_password_elements[char_index]);
                    temp_password = pass_array.iter().collect(); //storing temp_password in a string for ease of displaying and writing to file

                    if double_click_state == 1 {
                        //double click is to move to the next menu. Double click is canceled if any key other than select is clicked
                        double_click_state = 0;
                    }
                }
                previous_scroll_down_button_state = 1;
            
            }
            while scroll_down_button.is_high() && previous_scroll_down_button_state == 1 {
                previous_scroll_down_button_state = 0;
            }

            while !select_button.is_high() {
                // we either press the select button to move to the next menu or select a character for the temp_password
                if previous_select_button_state == 0 {
     
                    select_button_state = 1;
                    _pass_index += 1;
                    char_index = 0;

                    if double_click_state == 0 {
                        //initially it is zero and is set to 1 to be able to select multiple characters as required by the temp_password
                        double_click_state = 1;
                    } else if double_click_state == 1 {
                        // if the double click state is already 1 and its pressed again then it moves on to the next menu
                        password = temp_password.clone();
                        menu_state = 7;
                    }
                }
                previous_select_button_state = 1;
              
            }
            while select_button.is_high() && previous_select_button_state == 1 {
                previous_select_button_state = 0;
            }
            while !back_button.is_high() {
                //this button is used either to go back to the previous state or delete a selected character
                if previous_back_button_state == 0 {

                    _back_button_state = 1;

                    if pass_array.len() > 0 {
                        //if this button is pressed and the pass_array is greater than one, it removes the latest character
                        pass_array.remove(pass_array.len() - 1);
                    } else if _back_button_state == 1 {
                        //else it goes back to the previous menu
                        menu_state = 5;
                    }
                    _back_button_state = 0;

                    if double_click_state == 1 {
                        double_click_state = 0;
                    }
                }
                previous_back_button_state = 1;
              
            }
            while back_button.is_high() && previous_back_button_state == 1 {
                previous_back_button_state = 0;
            }
            while !scroll_up_button.is_high() {
                if previous_scroll_up_button_state == 0 {

                    if select_button_state == 1 {
                        //If a character is selected it is pushed onto the vector array here
                        pass_array.push(temp_password_elements[char_index]);
                        select_button_state = 0;
                    }
                    char_index -= 1;
                    if char_index == 0 {
                        //if last character reached, start again from the first character
                        char_index = 93;
                    }

                    if pass_array.len() > 0 {
                        // replaces the display character every cycle to make sure the latest character is being displayed
                        pass_array.remove(pass_array.len() - 1);
                    }
                    pass_array.push(temp_password_elements[char_index]);
                    temp_password = pass_array.iter().collect(); //storing temp_password in a string for ease of displaying and writing to file

                    if double_click_state == 1 {
                        //double click is to move to the next menu. Double click is canceled if any key other than select is clicked
                        double_click_state = 0;
                    }
                }
                previous_scroll_up_button_state = 1;
              
            }
            while scroll_up_button.is_high() && previous_scroll_up_button_state == 1 {
                previous_scroll_up_button_state = 0;
            }
        } else if menu_state == 7 {

            _save_password_state = 0; // just for checking and moving on to the next functionality
            while !select_button.is_high()
            { 
                while previous_select_button_state == 0
                {
                    
                    _save_password_state = 1;
                    temp_password = String::from("_");
                    pass_array = vec!['_'];
                    connect_wifi(all_ssids[ssid_index].as_str(), &password);
                    println!("entered function");
                    menu_state = 1;
                    break;
                }
                previous_select_button_state = 1;
        
            }
            
            if _save_password_state == 1 {
                println!("Entered file writing loop");
                password_file
                    .write(all_ssids[ssid_index].as_str().as_bytes())
                    .expect("write failed");

                password_file.write(": ".as_bytes()).expect("write failed");

                password_file
                    .write(password.as_bytes())
                    .expect("write failed");

                password_file.write("\n".as_bytes()).expect("write failed");
                // save_password_state = 1;
            }

            while  select_button.is_high()  &&  previous_select_button_state == 1 {
                previous_select_button_state = 0;
                
            }
        } 
        disp.clear();
        // disp.flush().unwrap();
        //--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
    }
    disp.clear();
    disp.flush().unwrap();
     // Ensure logs are flushed to the file before the program exits
     log::logger().flush();
}

fn get_wifi_ssids() -> Vec<String> {
    // Execute iwlist command to get available WiFi networks
    let output = Command::new("bash")
        .arg("-c")
        .arg("sudo iwlist wlan0 scan | grep 'ESSID:\"' | cut -d '\"' -f 2")
        .output()
        .expect("Failed to execute command");

    if output.status.success() {
        // Extract SSID names
        if let Ok(output_str) = String::from_utf8(output.stdout) {
            return output_str
                .lines()
                .map(|ssid| ssid.trim().to_string())
                .collect();
        }
    }

    eprintln!("Error executing iwlist command");
    Vec::new()
}

fn connect_wifi(ssid: &str, password: &str) {
    let output = Command::new("bash")
        .args(&[
            "-c",
            &format!(
                "sudo nmcli device wifi connect {} password {}",
                ssid, password
            ),
        ])
        .output()
        .expect("Failed to execute command");

    if output.status.success() {
        if let Ok(output_str) = String::from_utf8(output.stdout) {
            println!("{}", output_str);
        }
       
    } else {
        let error_str = String::from_utf8_lossy(&output.stderr);
        eprintln!("Command failed with error: {}", error_str);
        // println!("The error message displayed on connect wifi is : {}",error_str);
    }

}

