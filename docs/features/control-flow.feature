Feature: Control Flow and Logic
  As a young programmer
  I want to use loops and conditions
  So that I can create dynamic and interactive programs

  Background:
    Given I have an Ellex REPL session
    And the session is configured with default settings

  Scenario: Simple counting with repeat
    When I write the following program:
      """
      repeat 3 times do
          tell "Counting!"
      end
      """
    Then I should see the output "Counting!" exactly 3 times
    And the execution should complete successfully

  Scenario: Conditional logic with when
    Given I have set the variable "age" to 16
    When I write the following program:
      """
      when age is 16 do
          tell "You can drive!"
      end
      """
    Then I should see "You can drive!"

  Scenario: Conditional with otherwise clause
    Given I have set the variable "color" to "green"
    When I write the following program:
      """
      when color is "blue" do
          tell "Blue like the ocean!"
      otherwise do
          tell "That's a different color!"
      end
      """
    Then I should see "That's a different color!"

  Scenario: Nested loops
    When I write the following program:
      """
      repeat 2 times do
          repeat 2 times do
              tell "Nested!"
          end
      end
      """
    Then I should see the output "Nested!" exactly 4 times

  Scenario: Combining variables and loops
    Given I have set the variable "count" to 3
    When I write the following program:
      """
      repeat count times do
          tell "Loop number {count}"
      end
      """
    Then I should see "Loop number 3" exactly 3 times

  Scenario: Safety limits on loops
    When I write 'repeat 50000 times do tell "Too much!" end'
    Then I should see a friendly error message about loop limits
    And the execution should be safely terminated
    And I should see a suggestion to use fewer iterations