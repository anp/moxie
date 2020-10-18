// ***********************************************
// All of these tests are written to implement
// the official TodoMVC tests written for Selenium.
//
// The Cypress tests cover the exact same functionality,
// and match the same test names as TodoMVC.
//
// You can find the original TodoMVC tests here:
// https://github.com/tastejs/todomvc/blob/master/tests/test.js
//
// Copied from https://github.com/cypress-io/cypress-example-todomvc
// on 2020-08-15 under MIT license.
// ***********************************************

describe('TodoMVC - Moxie DOM', function () {
  // setup these constants to match what TodoMVC does
  let TODO_ITEM_ONE = 'buy some cheese'
  let TODO_ITEM_TWO = 'feed the cat'
  let TODO_ITEM_THREE = 'book a doctors appointment'

  beforeEach(function () {
    // By default Cypress will automatically
    // clear the Local Storage prior to each
    // test which ensures no todos carry over
    // between tests.
    //
    // Go out and visit our local web server
    // before each test, which serves us the
    // TodoMVC App we want to test against
    cy.visit('/index.html')
    // TODO(#182) delete above line and uncomment below line
    // cy.visit('/')
  })

  afterEach(() => {
    // In firefox, blur handlers will fire upon navigation if there is an activeElement.
    // Since todos are updated on blur after editing,
    // this is needed to blur activeElement after each test to prevent state leakage between tests.
    cy.window().then((win) => {
      // @ts-ignore
      win.document.activeElement.blur()
    })
  })

  // a very simple example helpful during presentations
  it('adds 2 todos', function () {
    cy.get('.new-todo')
      .type('learn testing{enter}')
      .type('be cool{enter}')

    cy.get('.todo-list li').should('have.length', 2)
  })

  // TODO(https://github.com/cypress-io/cypress/issues/6207) uncomment
  // context('When page is initially opened', function () {
  //   it('should focus on the todo input field', function () {
  //     cy.get('.new-todo').should('have.focus')
  //   })
  // })

  context('No Todos', function () {
    it('should hide #main and #footer', function () {
      cy.get('.todo-list li').should('not.exist')
      cy.get('.main').should('not.exist')
      cy.get('.footer').should('not.exist')
    })
  })

  context('New Todo', function () {
    it('should allow me to add todo items', function () {
      // create 1st todo
      cy.get('.new-todo')
        .type(TODO_ITEM_ONE)
        .type('{enter}')

      // make sure the 1st label contains the 1st todo text
      cy.get('.todo-list li')
        .eq(0)
        .find('label')
        .should('contain', TODO_ITEM_ONE)

      // create 2nd todo
      cy.get('.new-todo')
        .type(TODO_ITEM_TWO)
        .type('{enter}')

      // make sure the 2nd label contains the 2nd todo text
      cy.get('.todo-list li')
        .eq(1)
        .find('label')
        .should('contain', TODO_ITEM_TWO)
    })

    it('adds items', function () {
      // create several todos then check the number of items in the list
      cy.get('.new-todo')
        .type('todo A{enter}')
        .type('todo B{enter}')
        .type('todo C{enter}')
        .type('todo D{enter}')

      cy.get('.todo-list li').should('have.length', 4)
    })

    it('should clear text input field when an item is added', function () {
      cy.get('.new-todo')
        .type(TODO_ITEM_ONE)
        .type('{enter}')

      cy.get('.new-todo').should('have.text', '')
    })

    it('should append new items to the bottom of the list', function () {
      // defined in cypress/support/commands.js
      cy.createDefaultTodos().as('todos')

      cy.get('.todo-count').contains('3 items left')

      cy.get('@todos')
        .eq(0)
        .find('label')
        .should('contain', TODO_ITEM_ONE)

      cy.get('@todos')
        .eq(1)
        .find('label')
        .should('contain', TODO_ITEM_TWO)

      cy.get('@todos')
        .eq(2)
        .find('label')
        .should('contain', TODO_ITEM_THREE)
    })

    it('should trim text input', function () {
      cy.createTodo(`    ${TODO_ITEM_ONE}    `)

      // we use as explicit assertion here about the text instead of
      // using 'contain' so we can specify the exact text of the element
      // does not have any whitespace around it
      cy.get('.todo-list li')
        .eq(0)
        .should('have.text', TODO_ITEM_ONE)
    })

    it('should show #main and #footer when items added', function () {
      cy.createTodo(TODO_ITEM_ONE)
      cy.get('.main').should('be.visible')
      cy.get('.footer').should('be.visible')
    })
  })

  context('Mark all as completed', function () {

    beforeEach(function () {
      cy.createDefaultTodos().as('todos')
    })

    it('should allow me to mark all items as completed', function () {
      // we use 'check' instead of 'click' because that indicates our intention much clearer
      cy.get('.toggle-all').check()

      // get each todo li and ensure its class is 'completed'
      cy.get('@todos')
        .eq(0)
        .should('have.class', 'completed')

      cy.get('@todos')
        .eq(1)
        .should('have.class', 'completed')

      cy.get('@todos')
        .eq(2)
        .should('have.class', 'completed')
    })

    it('should allow me to clear the complete state of all items', function () {
      // check and then immediately uncheck
      cy.get('.toggle-all')
        .check()
        .uncheck()

      cy.get('@todos')
        .eq(0)
        .should('not.have.class', 'completed')

      cy.get('@todos')
        .eq(1)
        .should('not.have.class', 'completed')

      cy.get('@todos')
        .eq(2)
        .should('not.have.class', 'completed')
    })

    it('complete all checkbox should update state when items are completed / cleared', function () {
      // alias the .toggle-all for reuse later
      cy.get('.toggle-all')
        .as('toggleAll')
        .check()
        .should('be.checked')

      // alias the first todo and then click it
      cy.get('.todo-list li')
        .eq(0)
        .as('firstTodo')
        .find('.toggle')
        .uncheck()

      // reference the .toggle-all element again
      // and make sure its not checked
      cy.get('@toggleAll').should('not.be.checked')

      // reference the first todo again and now toggle it
      cy.get('@firstTodo')
        .find('.toggle')
        .check()

      // assert the toggle all is checked again
      cy.get('@toggleAll').should('be.checked')
    })
  })

  context('Item', function () {
    it('should allow me to mark items as complete', function () {
      // the return value is the <li> in the <ul.todos-list>
      cy.createTodo(TODO_ITEM_ONE).as('firstTodo')
      cy.createTodo(TODO_ITEM_TWO).as('secondTodo')

      cy.get('@firstTodo')
        .find('.toggle')
        .check()

      cy.get('@firstTodo').should('have.class', 'completed')

      cy.get('@secondTodo').should('not.have.class', 'completed')
      cy.get('@secondTodo')
        .find('.toggle')
        .check()

      cy.get('@firstTodo').should('have.class', 'completed')
      cy.get('@secondTodo').should('have.class', 'completed')
    })

    it('should allow me to un-mark items as complete', function () {
      cy.createTodo(TODO_ITEM_ONE).as('firstTodo')
      cy.createTodo(TODO_ITEM_TWO).as('secondTodo')

      cy.get('@firstTodo')
        .find('.toggle')
        .check()

      cy.get('@firstTodo').should('have.class', 'completed')
      cy.get('@secondTodo').should('not.have.class', 'completed')

      cy.get('@firstTodo')
        .find('.toggle')
        .uncheck()

      cy.get('@firstTodo').should('not.have.class', 'completed')
      cy.get('@secondTodo').should('not.have.class', 'completed')
    })

    it('should allow me to edit an item', function () {
      cy.createDefaultTodos().as('todos')

      cy.get('@todos')
        .eq(1)
        .as('secondTodo')
        // TODO: fix this, dblclick should
        // have been issued to label
        .find('label')
        .dblclick()

      // clear out the inputs current value
      // and type a new value
      cy.get('@secondTodo')
        .find('.edit')
        .clear()
        .type('buy some sausages')
        .type('{enter}')

      // explicitly assert about the text value
      cy.get('@todos')
        .eq(0)
        .should('contain', TODO_ITEM_ONE)

      cy.get('@secondTodo').should('contain', 'buy some sausages')
      cy.get('@todos')
        .eq(2)
        .should('contain', TODO_ITEM_THREE)
    })
  })

  context('Editing', function () {
    beforeEach(function () {
      cy.createDefaultTodos().as('todos')
    })

    it('should hide other controls when editing', function () {
      cy.get('@todos')
        .eq(1)
        .as('secondTodo')
        .find('label')
        .dblclick()

      cy.get('@secondTodo')
        .find('.toggle')
        .should('not.be.visible')

      cy.get('@secondTodo')
        .find('label')
        .should('not.be.visible')
    })

    it('should save edits on blur', function () {
      cy.get('@todos')
        .eq(1)
        .as('secondTodo')
        .find('label')
        .dblclick()

      cy.get('@secondTodo')
        .find('.edit')
        .clear()
        .type('buy some sausages')
        .blur()

      cy.get('@todos')
        .eq(0)
        .should('contain', TODO_ITEM_ONE)

      cy.get('@secondTodo').should('contain', 'buy some sausages')
      cy.get('@todos')
        .eq(2)
        .should('contain', TODO_ITEM_THREE)
    })

    it('should trim entered text', function () {
      cy.get('@todos')
        .eq(1)
        .as('secondTodo')
        .find('label')
        .dblclick()

      cy.get('@secondTodo')
        .find('.edit')
        .clear()
        .type('    buy some sausages    ')
        .type('{enter}')

      cy.get('@todos')
        .eq(0)
        .should('contain', TODO_ITEM_ONE)

      cy.get('@secondTodo').should('contain', 'buy some sausages')
      cy.get('@todos')
        .eq(2)
        .should('contain', TODO_ITEM_THREE)
    })

    it('should remove the item if an empty text string was entered', function () {
      cy.get('@todos')
        .eq(1)
        .as('secondTodo')
        .find('label')
        .dblclick()

      cy.get('@secondTodo')
        .find('.edit')
        .clear()
        .type('{enter}')

      cy.get('@todos').should('have.length', 2)
    })

    it('should cancel edits on escape', function () {
      cy.get('@todos')
        .eq(1)
        .as('secondTodo')
        .find('label')
        .dblclick()

      cy.get('@secondTodo')
        .find('.edit')
        .clear()
        .type('foo{esc}')

      cy.get('@todos')
        .eq(0)
        .should('contain', TODO_ITEM_ONE)

      cy.get('@todos')
        .eq(1)
        .should('contain', TODO_ITEM_TWO)

      cy.get('@todos')
        .eq(2)
        .should('contain', TODO_ITEM_THREE)
    })
  })

  context('Counter', function () {
    it('should display the current number of todo items', function () {
      cy.createTodo(TODO_ITEM_ONE)
      cy.get('.todo-count').contains('1 item left')
      cy.createTodo(TODO_ITEM_TWO)
      cy.get('.todo-count').contains('2 items left')
    })
  })

  context('Clear completed button', function () {
    beforeEach(function () {
      cy.createDefaultTodos().as('todos')
    })

    it('should display the correct text', function () {
      cy.get('@todos')
        .eq(0)
        .find('.toggle')
        .check()

      cy.get('.clear-completed').contains('Clear completed')
    })

    it('should remove completed items when clicked', function () {
      cy.get('@todos')
        .eq(1)
        .find('.toggle')
        .check()

      cy.get('.clear-completed').click()
      cy.get('@todos').should('have.length', 2)
      cy.get('@todos')
        .eq(0)
        .should('contain', TODO_ITEM_ONE)

      cy.get('@todos')
        .eq(1)
        .should('contain', TODO_ITEM_THREE)
    })

    it('should be hidden when there are no items that are completed', function () {
      cy.get('@todos')
        .eq(1)
        .find('.toggle')
        .check()

      cy.get('.clear-completed')
        .should('be.visible')
        .click()

      cy.get('.clear-completed').should('not.exist')
    })
  })

  context('Persistence', function () {
    it('should persist its data', function () {
      function testState() {
        cy.get('@firstTodo')
          .should('contain', TODO_ITEM_ONE)
          .and('have.class', 'completed')

        cy.get('@secondTodo')
          .should('contain', TODO_ITEM_TWO)
          .and('not.have.class', 'completed')
      }

      cy.createTodo(TODO_ITEM_ONE).as('firstTodo')
      cy.createTodo(TODO_ITEM_TWO).as('secondTodo')
      cy.get('@firstTodo')
        .find('.toggle')
        .check()
        .then(testState)

        .reload()
        .then(testState)
    })
  })

  context('Routing', function () {
    beforeEach(function () {
      cy.createDefaultTodos().as('todos')
    })

    it('should allow me to display active items', function () {
      cy.get('@todos')
        .eq(1)
        .find('.toggle')
        .check()

      cy.get('.filters')
        .contains('Active')
        .click()

      cy.get('@todos')
        .eq(0)
        .should('contain', TODO_ITEM_ONE)

      cy.get('@todos')
        .eq(1)
        .should('contain', TODO_ITEM_THREE)
    })

    it('should respect the back button', function () {
      cy.get('@todos')
        .eq(1)
        .find('.toggle')
        .check()

      cy.get('.filters')
        .contains('Active')
        .click()

      cy.get('.filters')
        .contains('Completed')
        .click()

      cy.get('@todos').should('have.length', 1)
      cy.go('back')
      cy.get('@todos').should('have.length', 2)
      cy.go('back')
      cy.get('@todos').should('have.length', 3)
    })

    it('should allow me to display completed items', function () {
      cy.get('@todos')
        .eq(1)
        .find('.toggle')
        .check()

      cy.get('.filters')
        .contains('Completed')
        .click()

      cy.get('@todos').should('have.length', 1)
    })

    it('should allow me to display all items', function () {
      cy.get('@todos')
        .eq(1)
        .find('.toggle')
        .check()

      cy.get('.filters')
        .contains('Active')
        .click()

      cy.get('.filters')
        .contains('Completed')
        .click()

      cy.get('.filters')
        .contains('All')
        .click()

      cy.get('@todos').should('have.length', 3)
    })

    it('should highlight the currently applied filter', function () {
      // using a within here which will automatically scope
      // nested 'cy' queries to our parent element <ul.filters>
      cy.get('.filters').within(function () {
        cy.contains('All').should('have.class', 'selected')
        cy.contains('Active')
          .click()
          .should('have.class', 'selected')

        cy.contains('Completed')
          .click()
          .should('have.class', 'selected')
      })
    })
  })

  context('Contrast', () => {
    it('has good contrast when empty', () => {
      cy.addAxeCode()
      cy.checkA11y(null, {
        runOnly: ['cat.color'],
      })
    })

    it('has good contrast with several todos', () => {
      cy.addAxeCode()
      cy.get('.new-todo')
        .type('learn testing{enter}')
        .type('be cool{enter}')

      cy.get('.todo-list li').should('have.length', 2)
      cy.checkA11y(null, {
        runOnly: ['cat.color'],
      })

      // and after marking an item completed
      cy.get('.todo-list li')
        .first()
        .find('.toggle')
        .check()

      cy.get('.todo-list li')
        .first()
        .should('have.class', 'completed')

      cy.checkA11y(null, {
        runOnly: ['cat.color'],
      })
    })
  })
})
