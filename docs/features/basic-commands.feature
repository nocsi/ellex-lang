Feature: Basic Ellex Commands
  As a young programmer
  I want to use natural language commands
  So that I can create programs that are easy to understand

  Background:
    Given I have an Ellex REPL session
    And the session is configured with default settings

  Scenario: Saying hello to the world
    When I write 'tell "Hello, world!"'
    Then I should see the output "Hello, world!"
    And the execution should complete successfully

  Scenario: Getting user input
    When I write 'ask "What's your name?" → name'
    And I provide the input "Alice"
    Then the variable "name" should contain "Alice"
    And I should see "Stored 'Alice' in variable 'name'"

  Scenario: Using variables in messages
    Given I have set the variable "name" to "Bob"
    When I write 'tell "Hello, {name}!"'
    Then I should see the output "Hello, Bob!"

  Scenario: Working with numbers
    When I write 'tell 42'
    Then I should see the output "42"
    And the value should be recognized as a number

  Scenario: Creating and using lists
    When I write 'tell [1, 2, 3]'
    Then I should see the output "[1, 2, 3]"
    And the value should be recognized as a list

  Scenario: Using comments
    When I write '# This is a comment'
    Then I should see no output
    And the comment should be ignored during execution

  Scenario: Multi-line programs
    When I write the following program:
      """
      # Greet the user
      ask "What's your name?" → user_name
      tell "Nice to meet you, {user_name}!"
      """
    And I provide the input "Charlie"
    Then I should see "Nice to meet you, Charlie!"
    And the variable "user_name" should contain "Charlie"