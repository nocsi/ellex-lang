Feature: Modal Programming
  As a young programmer
  I want to use different modes for different types of interactions
  So that I can explore, understand, and build code more effectively

  Background:
    Given I have an Ellex REPL session with modal programming enabled
    And I have some example code loaded

  Scenario: Using Listen Mode for exploration
    When I enter listen mode with:
      """
      @listen do
          tell
      end
      """
    Then I should see documentation about the 'tell' command
    And I should see usage examples
    And I should see related commands

  Scenario: Using Think Mode for analysis
    Given I have a function "greet_user" defined
    When I enter think mode with:
      """
      @think do
          what does greet_user do?
      end
      """
    Then I should see an explanation of the greet_user function
    And I should see what variables it uses
    And I should see what output it produces

  Scenario: Using Build Mode for refactoring
    Given I have some repetitive code
    When I enter build mode with:
      """
      @build do
          extract lines 5-8 → new_function
      end
      """
    Then the selected lines should be extracted into a new function
    And the original code should call the new function
    And I should see "Function extracted successfully"

  Scenario: Using Teach Mode for learning
    When I enter teach mode with:
      """
      @teach do
          explain loops
      end
      """
    Then I should see a kid-friendly explanation of loops
    And I should see simple examples
    And I should see practice suggestions

  Scenario: Modal commands with AI assistance
    Given AI assistance is enabled
    When I enter think mode with:
      """
      @think do
          how can I make this code better?
      end
      """
    Then I should see AI-powered suggestions
    And I should see code improvement recommendations
    And I should see learning opportunities

  Scenario: Listen mode for exploring around functions
    Given I have a function "counting_game" defined
    When I enter listen mode with:
      """
      @listen do
          around counting_game
      end
      """
    Then I should see related functions
    And I should see similar patterns in my code
    And I should see suggested improvements

  Scenario: Build mode for renaming
    Given I have a variable "old_name" in my code
    When I enter build mode with:
      """
      @build do
          rename old_name → new_name
      end
      """
    Then all occurrences of "old_name" should be renamed to "new_name"
    And I should see "Renamed old_name to new_name in 3 places"

  Scenario: Teach mode with specific topics
    When I enter teach mode with:
      """
      @teach do
          show examples of functions
      end
      """
    Then I should see multiple function examples
    And I should see explanations of when to use functions
    And I should see progressive complexity examples

  Scenario: Nested modal usage
    When I enter listen mode and then think mode:
      """
      @listen do
          tell
      end
      @think do
          when should I use tell?
      end
      """
    Then I should see both exploration and analysis results
    And the modes should work together seamlessly