use crate::interpreter::interpreter::Interpreter;
use crate::parser::ast::Stmt;
use crate::values::values::{Value, Function, ControlFlow};
use crate::interpreter::error::{RuntimeError, RuntimeResult};
use std::collections::HashMap;

impl Interpreter
{
    pub(crate) fn execute(&mut self, stmt: &Stmt) -> RuntimeResult<()> {
        match stmt {
            Stmt::Expression(expr) => {
                let _ = self.eval(expr)?;
                Ok(())
            }

            Stmt::SmartLock { variable } => {
                let value = self.environment.get(variable)?;
                self.environment.define_smart_lock(variable, value)?;
                Ok(())
            }

            Stmt::SmartUnlock { variable } => {
                let value = self.environment.get(variable)?;
                self.environment.define_smart_unclock(variable, value)?;
                Ok(())
            }

            Stmt::SmartKill { variable } => {
                self.environment.delete_variable(variable)?;
                Ok(())
            }

            Stmt::SmartRevive { variable } => {
                self.environment.define(variable, Value::Nil)?;
                Ok(())
            }

            Stmt::SmartConst { variable } => {
                let value = self.environment.get(variable)?;
                self.environment.define_constant(variable, value)?;
                Ok(())
            }

            Stmt::If {
                condition,
                then_branch,
                elif_branches,
                else_branch,
            } => {
                // Evaluate main if condition
                let condition_value = self.eval(condition)?;

                if condition_value.truthy() {
                    // Execute then branch
                    for stmt in then_branch {
                        self.execute(stmt)?;
                    }
                } else {
                    // Check elif branches
                    let mut executed = false;

                    for (elif_condition, elif_body) in elif_branches {
                        let elif_value = self.eval(elif_condition)?;

                        if elif_value.truthy() {
                            for stmt in elif_body {
                                self.execute(stmt)?;
                            }
                            executed = true;
                            break;
                        }
                    }

                    // Execute else branch if no elif was executed
                    #[allow(clippy::collapsible_if)]
                    if !executed {
                        if let Some(else_body) = else_branch {
                            for stmt in else_body {
                                self.execute(stmt)?;
                            }
                        }
                    }
                }

                Ok(())
            }

            Stmt::Label { _label_ } => {
                for label_item in _label_ {
                    let label_name = label_item.0.clone();
                    let is_callable = label_item.1;
                    let visible = label_item.2.clone();
                    let params = label_item.3.clone();
                    let args = label_item.4.clone();
                    let body = label_item.5.clone();

                    if is_callable {
                        // Store callable label as function in environment
                        let func = Value::Function(Function {
                            name: label_name.clone(),
                            params,
                            defaults: args,
                            body,
                            visible_blocks: visible,
                        });
                        self.environment.define(&label_name, func)?;
                    } else {
                        // Store control flow label in environment
                        let ctrl = Value::ControlFlow(ControlFlow {
                            name: label_name.clone(),
                            body,
                        });
                        self.environment.define(&label_name, ctrl)?;
                    }
                }
                Ok(())
            }
            Stmt::Jump { jump } => {
                // Get the target label from environment
                let target_value = self.environment.get(jump)?;

                match target_value {
                    Value::ControlFlow(ctrl) => {
                        // Execute the control flow label's body
                        self.environment.push_scope();
                        for stmt in &ctrl.body {
                            self.execute(stmt)?;
                        }
                        self.environment.pop_scope();
                        Ok(())
                    }
                    _ => Err(RuntimeError::custom(format!(
                        "'{}' is not a valid jump target (must be a control flow label)",
                        jump
                    ))),
                }
            }

            Stmt::Pass => {
                // Do nothing - pass statement
                Ok(())
            }

            Stmt::For {
                iterator,
                iterable,
                body,
            } => {
                // Evaluate iterable expression
                let iter_val = self.eval(iterable)?;

                match iter_val {
                    Value::Array(items) => {
                        // For-loop runs in its own scope (optional but clean)
                        self.environment.push_scope();

                        for item in items {
                            // Each iteration can get its own nested scope (optional).
                            // If you want iterator variable to be updated in same scope, remove this push/pop.
                            self.environment.push_scope();

                            // Bind iterator variable
                            self.environment.define(iterator, item)?;

                            // Execute body
                            for stmt in body {
                                self.execute(stmt)?;
                            }

                            self.environment.pop_scope();
                        }

                        self.environment.pop_scope();
                        Ok(())
                    }

                    _ => Err(RuntimeError::custom(format!(
                        "For-loop expects an Array iterable, got {}",
                        iter_val.type_name()
                    ))),
                }
            }

            Stmt::While { condition, body } => {
                // Keep looping while condition is truthy
                while self.eval(condition)?.truthy() {
                    self.environment.push_scope();

                    for stmt in body {
                        self.execute(stmt)?;
                    }

                    self.environment.pop_scope();
                }
                Ok(())
            }

            Stmt::DoWhile { body, condition } => {
                // Execute body at least once
                loop {
                    self.environment.push_scope();

                    for stmt in body {
                        self.execute(stmt)?;
                    }

                    self.environment.pop_scope();

                    // Check condition after executing body
                    if !self.eval(condition)?.truthy() {
                        break;
                    }
                }
                Ok(())
            }

            Stmt::Visible { _name_, _block_ } => {
                // Visible blocks are NOT executed in global scope
                // They are only DEFINED here and will be initialized
                // when a function with permission first accesses them

                // Store the definition for later initialization
                self.visible_definitions
                    .insert(_name_.clone(), _block_.clone());

                // Mark as defined but not initialized
                self.visible.insert(_name_.clone(), HashMap::new());
                self.initialized_visible.insert(_name_.clone(), false);

                Ok(())
            }

        
        }
    }
}