Feature: REPL Interactive Commands
  As a young programmer
  I want to use special REPL commands
  So that I can manage my session and get help when needed

  Background:
    Given I have an Ellex REPL session
    And the session is configured with default settings

  Scenario: Getting help
    When I write '/help'
    Then I should see "🌿 Ellex REPL Help"
    And I should see basic command explanations
    And I should see REPL command explanations
    And I should see example code snippets

  Scenario: Setting variables directly
    When I write '/set name "Alice"'
    Then I should see 'Set name = "Alice"'
    And the variable "name" should contain "Alice"

  Scenario: Setting numeric variables
    When I write '/set age 15'
    Then I should see "Set age = 15"
    And the variable "age" should contain the number 15

  Scenario: Viewing all variables
    Given I have set variables:
      | name | value   |
      | user | "Bob"   |
      | age  | 16      |
    When I write '/vars'
    Then I should see "Variables:"
    And I should see "user = Bob"
    And I should see "age = 16"

  Scenario: Viewing variables when none exist
    When I write '/vars'
    Then I should see "No variables defined."

  Scenario: Viewing command history
    Given I have executed the commands:
      | command               |
      | tell "Hello!"         |
      | ask "Name?" → name    |
      | tell "Hi {name}!"     |
    When I write '/history'
    Then I should see "Command History:"
    And I should see all my previous commands in reverse order

  Scenario: Clearing output buffer
    Given I have some output in the buffer
    When I write '/clear'
    Then I should see "Output cleared."
    And the output buffer should be empty

  Scenario: Viewing current configuration
    When I write '/config'
    Then I should see "Configuration:"
    And I should see "Execution timeout: 5000ms"
    And I should see "Memory limit: 64MB"
    And I should see "Turtle graphics: enabled"
    And I should see "AI assistance: enabled"

  Scenario: Resetting the session
    Given I have variables and command history
    When I write '/reset'
    Then I should see "Session reset."
    And all variables should be cleared
    And the command history should be cleared
    And the execution count should be reset to 0

  Scenario: Viewing defined functions
    Given I have defined functions "greet" and "count"
    When I write '/funcs'
    Then I should see "Functions:"
    And I should see "greet"
    And I should see "count"

  Scenario: Exiting the REPL
    When I write '/exit'
    Then the REPL session should terminate gracefully
    And I should see "Goodbye! 👋"

  Scenario: Alternative exit commands
    When I write any of: '/quit', 'exit', 'quit'
    Then the REPL session should terminate gracefully