Feature: Function Definition and Calling
  As a young programmer
  I want to create reusable functions
  So that I can organize my code and avoid repetition

  Background:
    Given I have an Ellex REPL session
    And the session is configured with default settings

  Scenario: Defining a simple function
    When I write the following program:
      """
      make greet_user do
          ask "What's your name?" → name
          tell "Hello, {name}!"
      end
      """
    Then the function "greet_user" should be defined
    And I should see "Function greet_user defined"

  Scenario: Calling a defined function
    Given I have defined a function "greet_user" with:
      """
      make greet_user do
          tell "Hello from function!"
      end
      """
    When I write 'greet_user'
    Then I should see "Hello from function!"

  Scenario: Functions with multiple statements
    When I write the following program:
      """
      make counting_game do
          tell "Let's count together!"
          repeat 3 times do
              tell "Counting..."
          end
          tell "All done!"
      end
      """
    And I call the function 'counting_game'
    Then I should see:
      """
      Let's count together!
      Counting...
      Counting...
      Counting...
      All done!
      """

  Scenario: Calling undefined function
    When I write 'unknown_function'
    Then I should see a friendly error message
    And the error should suggest "Function 'unknown_function' is not defined"
    And the error should suggest "Use 'make unknown_function' to define it"

  Scenario: Function with conditional logic
    When I write the following program:
      """
      make check_age do
          ask "How old are you?" → user_age
          when user_age is 16 do
              tell "You can drive!"
          otherwise do
              tell "Not quite yet!"
          end
      end
      """
    And I call the function 'check_age'
    And I provide the input "18"
    Then I should see "You can drive!"

  Scenario: Multiple function definitions
    When I write the following program:
      """
      make say_hello do
          tell "Hello!"
      end
      
      make say_goodbye do
          tell "Goodbye!"
      end
      """
    Then both functions "say_hello" and "say_goodbye" should be defined
    And I can call 'say_hello' to see "Hello!"
    And I can call 'say_goodbye' to see "Goodbye!"